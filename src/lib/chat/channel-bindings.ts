export function isActiveConversationChannelBinding(bindingType: string | null | undefined) {
  return bindingType === "active" || bindingType === "chat";
}
