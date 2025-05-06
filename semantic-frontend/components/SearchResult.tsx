"use client";

import { SearchResultsProps, ServiceResult } from "../types";

export default function SearchResults({
  results,
  loading,
  query,
}: SearchResultsProps) {
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

  // Sort the results by a combination of distance and semantics
  const sortedResults = results.sort((a, b) => {
    // Weight the distance and similarity score
    const distanceWeight = 0.5;
    const similarityWeight = 0.5;
  
    // If both distances are available, calculate a score based on distance and similarity
    if (a.distance && b.distance && a.distance !== null && b.distance !== null) {
      const scoreA = (1 - a.similarity) * similarityWeight + (a.distance / 1000) * distanceWeight;
      const scoreB = (1 - b.similarity) * similarityWeight + (b.distance / 1000) * distanceWeight;
      return scoreA - scoreB;
    }
  
    // If only one distance is available, prioritize the one with the available distance
    if (a.distance === null && b.distance !== null) {
      return 1; // a comes after b
    }
    if (a.distance !== null && b.distance === null) {
      return -1; // a comes before b
    }
  
    // If neither distance is available, sort based on similarity score only
    return b.similarity - a.similarity;
  });

  return (
    <div className="max-w-4xl mx-auto">
      <h2 className="text-xl font-semibold mb-4">Results ({sortedResults.length})</h2>
      <div className="space-y-4">
        {sortedResults.map((service: ServiceResult) => <ServiceCard key={service.id} service={service} />)}
      </div>
    </div>
  );
}

function ServiceCard({ service }: { service: ServiceResult }) {
  const metersToMiles = (meters: number) => {
    return meters / 1609.34;
  };
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

        {service.distance != null && (
          <span className="px-2 py-1 bg-orange-100 text-orange-800 text-sm rounded">
            {metersToMiles(service.distance).toFixed(2)} miles
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
