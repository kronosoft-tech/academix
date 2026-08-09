export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface AIProvider {
  name: string;
  chat(messages: ChatMessage[]): AsyncGenerator<string>;
}
