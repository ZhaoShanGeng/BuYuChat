# RAG 知识库引擎

> 路径：`src-tauri/src/services/rag/`

## 目录结构

```
rag/
├── mod.rs          # RagService 入口
├── processor.rs    # 文档解析 + 分块
├── embedder.rs     # 向量嵌入（本地/远程）
├── vector_store.rs # VectorStore（HNSW + SQLite）
├── retriever.rs    # 检索
└── citation.rs     # 引用标注解析
```

---

## RagService（入口）

```rust
pub struct RagService {
    db: SqlitePool,
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<VectorStore>,
}

impl RagService {
    /// 导入文档（文件路径 或 URL）
    ///
    /// 1. INSERT documents 表，status='indexing'
    /// 2. 后台 tokio::spawn 执行：parse → chunk → embed → store
    /// 3. 完成后 UPDATE status='ready'，失败 UPDATE status='error', error_msg=...
    /// 4. 通过 Tauri emit "rag:document:status" 事件通知前端
    ///
    /// 返回：新建的 document id（前端用于轮询状态或订阅事件）
    pub async fn import_document(
        &self,
        source: DocumentSource,
        app_handle: AppHandle,
    ) -> Result<String>;

    /// 删除文档（同时删除 chunks 和 HNSW 索引中的向量）
    pub async fn delete_document(&self, doc_id: &str) -> Result<()>;

    /// 列出所有文档
    pub async fn list_documents(&self) -> Result<Vec<DocumentRow>>;

    /// 按文档状态过滤
    pub async fn list_documents_by_status(
        &self,
        status: &str,   // "ready" | "indexing" | "error"
    ) -> Result<Vec<DocumentRow>>;

    /// 检索（供 ChatService 调用）
    ///
    /// 对 query 向量化，在 knowledge_base_ids 关联的文档中搜索 top_k 最相似块
    /// 返回 Citation 列表（按相似度降序）
    pub async fn retrieve(
        &self,
        query: &str,
        knowledge_base_ids: &[String],
        top_k: usize,       // 通常 5
        min_score: f32,     // 相似度阈值，低于此值的结果丢弃；建议 0.5
    ) -> Result<Vec<Citation>>;

    /// URL 爬取（不存入知识库，直接返回正文供临时注入上下文）
    pub async fn scrape_url(&self, url: &str) -> Result<String>;
}

#[derive(Debug)]
pub enum DocumentSource {
    File { path: std::path::PathBuf, name: String },
    Url  { url: String },
}
```

---

## DocumentProcessor（文档解析与分块）

```rust
// src-tauri/src/services/rag/processor.rs

/// 解析器 trait
pub trait DocumentParser: Send + Sync {
    fn can_handle(&self, path: &Path) -> bool;
    async fn parse(&self, path: &Path) -> Result<Vec<ParsedPage>>;
}

pub struct ParsedPage {
    pub page_number: Option<u32>,  // PDF 有页码，其他 None
    pub text: String,
}

/// 各解析器
pub struct PdfParser;      // lopdf，提取文本（按页）
pub struct DocxParser;     // docx-rs，按段落提取
pub struct PlainTextParser;// 直接读取文件内容
pub struct WebScraper;     // scraper crate，提取 <article>/<main>/<body> 主要内容

/// 分块器
pub struct Chunker {
    pub chunk_size: usize,    // 默认 512 tokens（按字符估算：512 * 3.5 ≈ 1792 字符）
    pub overlap: usize,       // 默认 50 tokens（≈ 175 字符）
}

impl Chunker {
    /// 将文档文本分块
    /// 策略：按句子边界滑动窗口（用 '。' '.' '!' '?' 等断句）
    /// 不足一块时整体作为一块
    pub fn chunk(&self, pages: Vec<ParsedPage>) -> Vec<Chunk>;
}

pub struct Chunk {
    pub content: String,
    pub chunk_index: u32,
    pub page_number: Option<u32>,
}
```

---

## Embedder（向量嵌入）

```rust
// src-tauri/src/services/rag/embedder.rs
use async_trait::async_trait;

#[async_trait]
pub trait Embedder: Send + Sync {
    /// 对单个文本生成嵌入向量（归一化 L2）
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// 批量嵌入（实现可并发）；默认实现：逐条调用 embed()
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// 向量维度（本地模型 384，OpenAI text-embedding-3-small 1536）
    fn dimension(&self) -> usize;
}

/// 本地嵌入（ONNX Runtime，离线）
pub struct LocalEmbedder {
    session: ort::Session,  // all-MiniLM-L6-v2 ONNX 模型
    // 模型文件路径：app_data_dir/models/all-MiniLM-L6-v2.onnx
    // 首次启动时若文件不存在，提示用户下载或自动下载
}

/// 远程嵌入 API
pub struct ApiEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,     // 默认 "text-embedding-3-small"
    base_url: String,  // 默认 OpenAI，可覆盖
}
// embed() 实现：POST /embeddings，解析 data[0].embedding
```

---

## VectorStore（向量存储与检索）

```rust
// src-tauri/src/services/rag/vector_store.rs
use usearch::Index;
use tokio::sync::Mutex;

pub struct VectorStore {
    index: Mutex<Index>,  // HNSW 内存索引（usearch crate）
    db: SqlitePool,
    dimension: usize,
}

impl VectorStore {
    /// 创建并从 SQLite chunks 表加载已有向量（应用启动时调用）
    pub async fn load_from_db(db: &SqlitePool, dimension: usize) -> Result<Self>;

    /// 添加向量（同时写 SQLite 和内存索引）
    /// SQLite：INSERT INTO chunks (id, document_id, content, embedding, chunk_index, page_number)
    /// HNSW：index.add(numeric_key, &embedding)
    /// numeric_key 为 i64，从 SQLite 自增辅助表获取（或用 UUID 哈希）
    pub async fn add(
        &self,
        document_id: &str,
        chunks: Vec<Chunk>,
        embeddings: Vec<Vec<f32>>,
    ) -> Result<()>;

    /// 检索 top_k 最近邻
    /// HNSW 返回 numeric_key 列表 → 查 SQLite 获取 content 和元数据
    pub async fn search(
        &self,
        query_embedding: &[f32],
        filter_doc_ids: &[String],  // 限定在这些文档中搜索；空则全库搜索
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;

    /// 删除文档的所有向量（SQLite DELETE + HNSW 标记删除）
    pub async fn delete_document(&self, document_id: &str) -> Result<()>;
}

pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub chunk_index: u32,
    pub page_number: Option<u32>,
    pub content: String,
    pub score: f32,  // 余弦相似度，0-1
}
```

---

## CitationEngine（引用标注解析）

```rust
// src-tauri/src/services/rag/citation.rs

/// 将 RAG 注入的 Citation 列表和 LLM 回复配对
///
/// LLM 回复约定包含 [1] [2] 格式的引用标记
/// 此函数提取回复中出现的引用编号，映射到对应 Citation
///
/// # 参数
/// - `response_text`：LLM 生成的原始文本
/// - `available_citations`：RAG 检索时提供给 LLM 的 Citation 列表（index 从 1 开始）
///
/// # 返回
/// - 清洗后的文本（保留 [n] 标记，前端渲染为可点击角标）
/// - 实际被引用的 Citation 子集
pub fn parse_citations(
    response_text: &str,
    available_citations: &[Citation],
) -> (String, Vec<Citation>);

// 实现：用正则 r"\[(\d+)\]" 找所有引用编号，过滤有效范围的，去重后返回
```

---

## 完整导入流程（后台任务）

```
DocumentSource（File/Url）
    │
    ▼  DocumentProcessor
解析 → Vec<ParsedPage>
    │
    ▼  Chunker
分块 → Vec<Chunk>（默认 512 tokens，50 overlap）
    │
    ▼  Embedder.embed_batch()
向量 → Vec<Vec<f32>>
    │
    ▼  VectorStore.add()
写入 SQLite chunks 表 + HNSW 内存索引
    │
    ▼  UPDATE documents SET status='ready', chunk_count=N
    │
    ▼  app_handle.emit("rag:document:status", {id, status:"ready"})
```

**性能预期（本地 CPU）：**
- 1 MB 文档（~500 chunks）嵌入时间：约 2-5 秒
- 检索延迟（1M 向量库）：< 50ms
