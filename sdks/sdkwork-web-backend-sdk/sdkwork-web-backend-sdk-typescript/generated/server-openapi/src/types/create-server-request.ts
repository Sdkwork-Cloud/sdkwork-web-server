export interface CreateServerRequest {
  name: string;
  host: string;
  sshPort: number;
  sshUser?: string;
  sshKeyPath?: string;
  description?: string;
}
