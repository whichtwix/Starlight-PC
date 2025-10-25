import { PUBLIC_API_URL } from "$env/static/public";
import { z } from "zod";

// ============================================================================
// Schemas
// ============================================================================

const NewsItemSchema = z.object({
  id: z.number(),
  title: z.string(),
  author: z.string(),
  content: z.string(),
  created_at: z.number(),
  updated_at: z.number(),
});

export type NewsItem = z.infer<typeof NewsItemSchema>;

// ============================================================================
// Query Functions
// ============================================================================

async function fetchWithValidation<T>(
  url: string,
  schema: z.ZodSchema<T>,
): Promise<T> {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`HTTP error: ${response.statusText}`);
  }

  const jsonData = await response.json();
  return schema.parse(jsonData);
}

export async function fetchNews(): Promise<NewsItem[]> {
  return await fetchWithValidation(
    `${PUBLIC_API_URL}/news`,
    z.array(NewsItemSchema),
  );
}

export function fetchNewsById(id: string | number): Promise<NewsItem> {
  return fetchWithValidation(`${PUBLIC_API_URL}/news/${id}`, NewsItemSchema);
}
