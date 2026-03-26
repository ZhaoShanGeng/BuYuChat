export type Notice = {
  kind: "success" | "error" | "info";
  text: string;
};

export type ChannelFormState = {
  name: string;
  baseUrl: string;
  apiKey: string;
  authType: string;
  modelsEndpoint: string;
  chatEndpoint: string;
  streamEndpoint: string;
  thinkingTagsInput: string;
  channelType: string;
  enabled: boolean;
};

export type SelectOption = {
  value: string;
  label: string;
};
