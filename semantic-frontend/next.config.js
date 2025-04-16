/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  experimental: {
    // Enable server components
    serverActions: true,
  },
}

module.exports = nextConfig