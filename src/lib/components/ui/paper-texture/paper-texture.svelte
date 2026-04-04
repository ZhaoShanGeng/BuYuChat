<script lang="ts">
  import { cn } from "$lib/utils";

  type Props = {
    class?: string;
    opacity?: number;
  };

  const { class: cls, opacity = 0.03 }: Props = $props();
</script>

<!-- 微妙的纸张颗粒感 overlay -->
<div
  class={cn("absolute inset-0 pointer-events-none mix-blend-multiply z-0", cls)}
  style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noiseFilter%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.8%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noiseFilter)%22/%3E%3C/svg%3E'); opacity: {opacity};"
></div>
