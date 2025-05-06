"use client";

import { useState } from "react";
import { SearchFormProps } from "../types";

export default function SearchForm({ onSearch, loading }: SearchFormProps) {
  const [query, setQuery] = useState("");
  const [location, setLocation] = useState<{ lat: string; lng: string }>({
    lat: "46.602248",
    lng: "-120.506076",
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    console.log("test")
    
    // Convert coordinates to numbers if provided
    const lat = location.lat ? parseFloat(location.lat) : undefined;
    const lng = location.lng ? parseFloat(location.lng) : undefined;
    
    onSearch(query, lat, lng);
  };

  return (
    <form onSubmit={handleSubmit} className="max-w-2xl mx-auto">
      <div className="space-y-4">
        {/* Search Input */}
        <div className="flex gap-2">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Describe what you're looking for..."
            className="flex-1 p-3 border border-gray-300 rounded shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            disabled={loading || !query.trim()}
            className={`px-6 py-3 bg-blue-600 text-white rounded shadow hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 ${
              loading || !query.trim() ? "opacity-70 cursor-not-allowed" : ""
            }`}
          >
            {loading ? "Searching..." : "Search"}
          </button>
        </div>

        {/* Location Inputs */}
        <div className="flex gap-2">
          <input
            type="number"
            step="any"
            value={location.lat}
            onChange={(e) => setLocation({ ...location, lat: e.target.value })}
            placeholder="Latitude (optional)"
            className="flex-1 p-2 border border-gray-300 rounded shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
          <input
            type="number"
            step="any"
            value={location.lng}
            onChange={(e) => setLocation({ ...location, lng: e.target.value })}
            placeholder="Longitude (optional)"
            className="flex-1 p-2 border border-gray-300 rounded shadow-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>
        
        <p className="text-sm text-gray-500">
          Tip: You can find coordinates using{" "}
          <a 
            href="https://www.latlong.net/" 
            target="_blank"
            className="text-blue-600 hover:underline"
          >
            latlong.net
          </a>
        </p>
      </div>
    </form>
  );
}