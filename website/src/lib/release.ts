import { queryOptions } from '@tanstack/react-query'
import { createServerFn } from '@tanstack/react-start'

export interface LatestRelease {
  version: string
  url: string
  pubDate: string | null
}

export const FALLBACK_VERSION = '0.3.1'
export const FALLBACK_DOWNLOAD_URL = `https://github.com/cowboyshibuya/anastasia/releases/download/v${FALLBACK_VERSION}/Anastasia-${FALLBACK_VERSION}.zip`

const APPCAST_URL =
  'https://github.com/cowboyshibuya/anastasia/releases/latest/download/appcast.xml'

// The Sparkle appcast has no CORS headers, so resolve it on the server.
const fetchLatestRelease = createServerFn({ method: 'GET' }).handler(
  async (): Promise<LatestRelease | null> => {
    try {
      const res = await fetch(APPCAST_URL, {
        signal: AbortSignal.timeout(3500),
        headers: {
          'User-Agent': 'Anastasia-Website',
        },
      })
      if (!res.ok) {
        return {
          version: FALLBACK_VERSION,
          url: FALLBACK_DOWNLOAD_URL,
          pubDate: null,
        }
      }
      const xml = await res.text()
      const version =
        xml.match(/sparkle:shortVersionString="([^"]+)"/)?.[1] ??
        xml.match(/<sparkle:shortVersionString>([^<]+)<\/sparkle:shortVersionString>/)?.[1] ??
        FALLBACK_VERSION

      const enclosureUrl =
        xml.match(/<enclosure[^>]+url="([^"]+)"/)?.[1] ??
        `https://github.com/cowboyshibuya/anastasia/releases/download/v${version}/Anastasia-${version}.zip`

      const pubDate = xml.match(/<pubDate>([^<]+)<\/pubDate>/)?.[1] ?? null

      return {
        version,
        url: enclosureUrl,
        pubDate,
      }
    } catch {
      return {
        version: FALLBACK_VERSION,
        url: FALLBACK_DOWNLOAD_URL,
        pubDate: null,
      }
    }
  },
)

export const releaseQuery = queryOptions({
  queryKey: ['latest-release'],
  queryFn: () => fetchLatestRelease(),
  staleTime: 5 * 60_000,
})
