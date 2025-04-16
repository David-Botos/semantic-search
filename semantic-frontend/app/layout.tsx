import './globals.css';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Semantic Service Search',
  description: 'Search for services using semantic vector search',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}