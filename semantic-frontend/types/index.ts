export interface ServiceResult {
  id: string;
  name: string;
  description?: string;
  short_description?: string;
  status: string;
  organization_name?: string;
  similarity: number;
  distance?: number; 
}

export interface SearchFormProps {
  onSearch: (query: string, lat?: number, lng?: number) => Promise<void>;
  loading: boolean;
}

export interface SearchResultsProps {
  results: ServiceResult[];
  loading: boolean;
  query: string;
}

export interface SearchRequest {
  query: string;
  limit?: number;
}

export interface SearchResponse {
  results: ServiceResult[];
}
