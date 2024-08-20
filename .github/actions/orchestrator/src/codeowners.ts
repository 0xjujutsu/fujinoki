/**
 * An individual entry from a CODEOWNERS file
 */
export interface CodeOwnersEntry {
  pattern: string;
  owners: string[];
}

/**
 * Parse a CODEOWNERS file into an array of entries (will be in reverse order
 * of the file).
 */
export function parseCodeOwners(str: string): CodeOwnersEntry[] {
  const entries: CodeOwnersEntry[] = [];
  const lines = str.split('\n');

  for (const line of lines) {
    const [content, comment] = line.split('#');
    const trimmed = content.trim();
    if (trimmed === '') continue;
    const [pattern, ...owners] = trimmed.split(/\s+/);
    entries.push({ pattern, owners });
  }

  return entries.reverse();
}
