import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import docsContent from '../docs/icpi-docs.md?raw'

export const Documentation: React.FC = () => {
  return (
    <div className="min-h-screen bg-[#000000] py-6">
      <div className="container px-3 max-w-5xl mx-auto">
        <div className="prose prose-invert max-w-none
          prose-headings:font-mono prose-headings:text-white
          prose-h1:text-3xl prose-h1:mb-4
          prose-h2:text-xl prose-h2:mt-8 prose-h2:mb-3
          prose-h3:text-base prose-h3:mt-4 prose-h3:mb-2
          prose-p:text-sm prose-p:font-mono prose-p:text-[#cccccc] prose-p:leading-relaxed
          prose-strong:text-white prose-strong:font-bold
          prose-code:text-[#00FF41] prose-code:font-mono prose-code:bg-transparent prose-code:px-1
          prose-pre:bg-[#000000] prose-pre:border prose-pre:border-[#1f1f1f] prose-pre:p-4 prose-pre:rounded
          prose-ul:text-sm prose-ul:font-mono prose-ul:text-[#cccccc] prose-ul:my-2
          prose-ol:text-sm prose-ol:font-mono prose-ol:text-[#cccccc] prose-ol:my-2
          prose-li:text-sm prose-li:my-1
          prose-hr:border-[#1f1f1f] prose-hr:my-8
          prose-em:text-[#999999] prose-em:italic
        ">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {docsContent}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  )
}
