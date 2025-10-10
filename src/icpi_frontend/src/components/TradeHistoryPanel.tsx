import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './ui/table'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { ScrollArea } from './ui/scroll-area'
import { Download, ChevronLeft, ChevronRight } from 'lucide-react'
import { RebalanceRecord, formatRebalanceAction, formatTradeTimestamp } from '../types/icpi'

interface TradeHistoryPanelProps {
  history: RebalanceRecord[]
}

export const TradeHistoryPanel: React.FC<TradeHistoryPanelProps> = ({ history }) => {
  const [currentPage, setCurrentPage] = useState(1)
  const ITEMS_PER_PAGE = 20

  const totalPages = Math.ceil(history.length / ITEMS_PER_PAGE)
  const startIdx = (currentPage - 1) * ITEMS_PER_PAGE
  const endIdx = startIdx + ITEMS_PER_PAGE
  const currentPageData = history.slice(startIdx, endIdx)

  const handleExport = () => {
    // Convert to CSV
    const csv = [
      ['Timestamp', 'Action', 'Token', 'Amount USD', 'Success', 'Details'].join(','),
      ...history.map(record => {
        const action = formatRebalanceAction(record.action)
        return [
          formatTradeTimestamp(record.timestamp),
          action.type,
          action.token,
          action.amount.toFixed(2),
          record.success ? 'Success' : 'Failed',
          `"${record.details.replace(/"/g, '""')}"`,  // Escape quotes in details
        ].join(',')
      })
    ].join('\n')

    const blob = new Blob([csv], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `icpi-trade-history-${Date.now()}.csv`
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <Card className="border-[#1f1f1f]">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">TRADE HISTORY</CardTitle>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleExport}
            className="text-xs"
            disabled={history.length === 0}
          >
            <Download className="h-3 w-3 mr-1" />
            EXPORT
          </Button>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <ScrollArea className="h-[400px]">
          <Table>
            <TableHeader>
              <TableRow className="border-[#1f1f1f]">
                <TableHead className="text-[10px] text-[#666666]">TIME</TableHead>
                <TableHead className="text-[10px] text-[#666666]">ACTION</TableHead>
                <TableHead className="text-[10px] text-[#666666]">TOKEN</TableHead>
                <TableHead className="text-[10px] text-[#666666]">AMOUNT</TableHead>
                <TableHead className="text-[10px] text-[#666666]">STATUS</TableHead>
                <TableHead className="text-[10px] text-[#666666]">DETAILS</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {currentPageData.length > 0 ? (
                currentPageData.map((record, idx) => {
                  const action = formatRebalanceAction(record.action)
                  const timestamp = formatTradeTimestamp(record.timestamp)

                  return (
                    <TableRow key={idx} className="border-[#1f1f1f] text-xs">
                      <TableCell className="font-mono text-[10px] text-[#999999]">
                        {new Date(timestamp).toLocaleTimeString()}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={action.type === 'buy' ? 'default' : action.type === 'sell' ? 'secondary' : 'outline'}
                          className="text-[10px]"
                        >
                          {action.type.toUpperCase()}
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-white">
                        {action.token}
                      </TableCell>
                      <TableCell className="font-mono text-[#999999]">
                        ${action.amount.toFixed(2)}
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-1">
                          <div className={`w-1.5 h-1.5 rounded-full ${
                            record.success ? 'bg-[#00FF41]' : 'bg-[#FF0055]'
                          }`} />
                          <span className={record.success ? 'text-[#00FF41]' : 'text-[#FF0055]'}>
                            {record.success ? 'Success' : 'Failed'}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell className="text-[10px] text-[#999999] max-w-[200px] truncate" title={record.details}>
                        {record.details}
                      </TableCell>
                    </TableRow>
                  )
                })
              ) : (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-[#666666] py-8">
                    No trade history yet
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </ScrollArea>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between p-3 border-t border-[#1f1f1f]">
            <div className="text-xs text-[#666666]">
              Page {currentPage} of {totalPages} ({history.length} total trades)
            </div>
            <div className="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage === 1}
              >
                <ChevronLeft className="h-3 w-3" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                disabled={currentPage === totalPages}
              >
                <ChevronRight className="h-3 w-3" />
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
