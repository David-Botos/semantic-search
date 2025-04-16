'use client';

import { useState } from 'react';
import SearchForm from '../components/SearchForm';
import SearchResults from '../components/SearchResult';
import { ServiceResult } from '../types';

export default function Home() {
  const [query, setQuery] = useState<string>('');
  const [results, setResults] = useState<ServiceResult[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  const handleSearch = async (searchQuery: string) => {
    if (!searchQuery.trim()) return;
    
    setLoading(true);
    setQuery(searchQuery);
    
    try {
      const response = await fetch('http://localhost:8080/search', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          query: searchQuery,
          limit: 10 
        }),
      });
      
      if (!response.ok) {
        throw new Error(`Error: ${response.status}`);
      }
      
      const data = await response.json();
      setResults(data);
    } catch (error) {
      console.error('Error searching:', error);
      setResults([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-gray-100 py-10 px-4">
      <div className="container mx-auto">
        <h1 className="text-3xl font-bold text-center mb-8">
          Semantic Service Search
        </h1>
        
        <SearchForm onSearch={handleSearch} loading={loading} />
        
        <div className="mt-8">
          <SearchResults 
            results={results} 
            loading={loading} 
            query={query} 
          />
        </div>
      </div>
    </main>
  );
}