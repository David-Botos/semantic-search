import { ServiceResult, SearchRequest } from '../types';

const API_URL = 'http://localhost:8080';

/**
 * Performs a semantic search using the Rust microservice
 */
export async function searchServices(query: string, limit: number = 10): Promise<ServiceResult[]> {
  if (!query.trim()) {
    return [];
  }

  const request: SearchRequest = {
    query,
    limit
  };

  const response = await fetch(`${API_URL}/search`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Search failed (${response.status}): ${errorText}`);
  }

  return await response.json();
}

/**
 * Checks if the search service is available
 */
export async function checkServiceStatus(): Promise<boolean> {
  try {
    const response = await fetch(`${API_URL}/`, {
      method: 'GET',
      cache: 'no-store',
    });
    
    return response.ok;
  } catch (error) {
    console.error('Error checking service status:', error);
    return false;
  }
}