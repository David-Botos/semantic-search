'use client';

import { SearchResultsProps, ServiceResult } from '../types';

export default function SearchResults({ results, loading, query }: SearchResultsProps) {
  if (loading) {
    return <p className="text-center text-gray-500 mt-8">Searching...</p>;
  }

  if (query && !loading && results.length === 0) {
    return (
      <p className="text-center text-gray-500 mt-8">
        No results found. Try a different search term.
      </p>
    );
  }

  if (results.length === 0) {
    return null;
  }

  return (
    <div className="max-w-4xl mx-auto">
      <h2 className="text-xl font-semibold mb-4">Results ({results.length})</h2>
      <div className="space-y-4">
        {results.map((service: ServiceResult) => (
          <ServiceCard key={service.id} service={service} />
        ))}
      </div>
    </div>
  );
}

function ServiceCard({ service }: { service: ServiceResult }) {
  return (
    <div className="bg-white p-6 rounded shadow">
      <h3 className="text-lg font-medium mb-2">{service.name}</h3>
      
      {service.description && (
        <p className="text-gray-600 mb-3">{service.description}</p>
      )}
      
      {service.short_description && !service.description && (
        <p className="text-gray-600 mb-3">{service.short_description}</p>
      )}
      
      <div className="flex flex-wrap gap-2">
        {service.organization_name && (
          <span className="px-2 py-1 bg-blue-100 text-blue-800 text-sm rounded">
            {service.organization_name}
          </span>
        )}
        
        <span className="px-2 py-1 bg-green-100 text-green-800 text-sm rounded">
          {service.status}
        </span>
        
        <span className="px-2 py-1 bg-purple-100 text-purple-800 text-sm rounded">
          Score: {Math.round(service.similarity * 100)}%
        </span>
      </div>
    </div>
  );
}