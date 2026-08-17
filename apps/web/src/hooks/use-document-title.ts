import { useEffect } from 'react'

export const ANASTASIA_DOCUMENT_TITLE = 'Waku Web'

export function formatDocumentTitle(section?: string | null): string {
  const normalized = section?.trim()
  if (!normalized || normalized === ANASTASIA_DOCUMENT_TITLE) return ANASTASIA_DOCUMENT_TITLE
  return `${normalized} — ${ANASTASIA_DOCUMENT_TITLE}`
}

export function useDocumentTitle(section?: string | null) {
  const title = formatDocumentTitle(section)
  useEffect(() => {
    document.title = title
  }, [title])
}
