import React from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './ui/table'
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts'
import { formatNumber } from '@/lib/utils'

export interface TokenAllocation {
  token: string
  value: number
  currentPercent: number
  targetPercent: number
  deviation: number
}

interface AllocationChartProps {
  data: TokenAllocation[]
  height?: number
}

const getTokenColor = (token: string): string => {
  const colors: Record<string, string> = {
    ALEX: '#8B5CF6',
    ZERO: '#3B82F6',
    KONG: '#F97316',
    BOB: '#10B981',
    ICP: '#29ABE2',
    ckUSDT: '#26A17B',
    CHAT: '#EC4899',
  }
  return colors[token] || '#888888'
}

const CustomTooltip = ({ active, payload }: any) => {
  if (active && payload && payload[0]) {
    const data = payload[0]
    return (
      <div className="bg-[#0a0a0a] p-2 border border-[#1f1f1f]">
        <p className="text-xs font-mono text-white">{data.name}</p>
        {data.payload.actualValue !== undefined && (
          <p className="text-xs font-mono text-[#999999]">
            {formatNumber(data.payload.actualValue)}
          </p>
        )}
        <p className="text-xs font-mono text-[#999999]">
          {data.value.toFixed(2)}%
        </p>
      </div>
    )
  }
  return null
}

export const AllocationChart: React.FC<AllocationChartProps> = ({
  data,
  height = 200
}) => {
  // Handle empty state during progressive loading
  if (!data || data.length === 0) {
    return (
      <Card className="border-[#1f1f1f]">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">PORTFOLIO ALLOCATION</CardTitle>
        </CardHeader>
        <CardContent className="p-3 space-y-3">
          {/* Skeleton for pie charts */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <div className="h-3 w-12 bg-[#1f1f1f] animate-pulse rounded mb-1" />
              <div className="flex items-center justify-center h-[200px]">
                <div className="w-32 h-32 rounded-full bg-[#1f1f1f] animate-pulse" />
              </div>
            </div>
            <div>
              <div className="h-3 w-12 bg-[#1f1f1f] animate-pulse rounded mb-1" />
              <div className="flex items-center justify-center h-[200px]">
                <div className="w-32 h-32 rounded-full bg-[#1f1f1f] animate-pulse" />
              </div>
            </div>
          </div>
          {/* Skeleton for table */}
          <div className="space-y-2">
            <div className="h-8 bg-[#1f1f1f] animate-pulse rounded" />
            {[1, 2, 3, 4].map(i => (
              <div key={i} className="h-10 bg-[#1f1f1f] animate-pulse rounded" />
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  const actualData = data.map(item => ({
    name: item.token,
    value: item.currentPercent,
    actualValue: item.value,
  }))

  const targetData = data.map(item => ({
    name: item.token,
    value: item.targetPercent,
  }))

  return (
    <Card className="border-[#1f1f1f]">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm">PORTFOLIO ALLOCATION</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 p-3">
        <div className="grid grid-cols-2 gap-3">
          <div>
            <p className="text-[10px] uppercase text-[#666666] mb-1 font-sans">Actual</p>
            <ResponsiveContainer width="100%" height={height}>
              <PieChart>
                <Pie
                  data={actualData}
                  dataKey="value"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={70}
                  label={({ name, value }) => `${name} ${value.toFixed(1)}%`}
                  labelLine={false}
                >
                  {actualData.map((entry, index) => (
                    <Cell key={`actual-${index}`} fill={getTokenColor(entry.name)} />
                  ))}
                </Pie>
                <Tooltip content={<CustomTooltip />} />
              </PieChart>
            </ResponsiveContainer>
          </div>

          <div>
            <p className="text-[10px] uppercase text-[#666666] mb-1 font-sans">Target</p>
            <ResponsiveContainer width="100%" height={height}>
              <PieChart>
                <Pie
                  data={targetData}
                  dataKey="value"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={70}
                  label={({ name, value }) => `${name} ${value.toFixed(1)}%`}
                  labelLine={false}
                >
                  {targetData.map((entry, index) => (
                    <Cell
                      key={`target-${index}`}
                      fill={getTokenColor(entry.name)}
                      opacity={0.5}
                    />
                  ))}
                </Pie>
                <Tooltip content={<CustomTooltip />} />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </div>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>TOKEN</TableHead>
              <TableHead className="text-right">VALUE</TableHead>
              <TableHead className="text-right">ACTUAL</TableHead>
              <TableHead className="text-right">TARGET</TableHead>
              <TableHead className="text-right">ΔVIATION</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map(token => (
              <TableRow key={token.token}>
                <TableCell>
                  <div className="flex items-center gap-1.5">
                    <div
                      className="w-2 h-2"
                      style={{ backgroundColor: getTokenColor(token.token) }}
                    />
                    <span className="font-sans">{token.token}</span>
                  </div>
                </TableCell>
                <TableCell className="text-right">{formatNumber(token.value)}</TableCell>
                <TableCell className="text-right">{token.currentPercent.toFixed(2)}%</TableCell>
                <TableCell className="text-right">{token.targetPercent.toFixed(2)}%</TableCell>
                <TableCell className={`text-right ${
                  Math.abs(token.deviation) < 1 ? 'text-[#00FF41]' :
                  Math.abs(token.deviation) < 3 ? 'text-[#FFE600]' : 'text-[#FF0055]'
                }`}>
                  {token.deviation > 0 ? '+' : ''}{token.deviation.toFixed(2)}%
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}