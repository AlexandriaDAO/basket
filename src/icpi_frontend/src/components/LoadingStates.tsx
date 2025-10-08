import React from 'react'
import { Card, CardContent } from './ui/card'
import { Skeleton } from './ui/skeleton'

export const PortfolioSkeleton = () => (
  <div className="space-y-4">
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {[...Array(4)].map((_, i) => (
        <Card key={i} className="glass-effect">
          <CardContent className="p-6">
            <Skeleton className="h-4 w-24 mb-2" />
            <Skeleton className="h-8 w-32" />
          </CardContent>
        </Card>
      ))}
    </div>
    <Card className="glass-effect">
      <CardContent className="p-6">
        <Skeleton className="h-64 w-full" />
      </CardContent>
    </Card>
  </div>
)

export const TableSkeleton = () => (
  <Card className="glass-effect">
    <CardContent className="p-6">
      <div className="space-y-3">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Skeleton className="h-8 w-8 rounded-full" />
              <Skeleton className="h-4 w-20" />
            </div>
            <Skeleton className="h-4 w-24" />
          </div>
        ))}
      </div>
    </CardContent>
  </Card>
)

export const ChartSkeleton = () => (
  <Card className="glass-effect">
    <CardContent className="p-6">
      <Skeleton className="h-4 w-32 mb-4" />
      <Skeleton className="h-[300px] w-full" />
      <div className="mt-4 space-y-2">
        {[...Array(3)].map((_, i) => (
          <div key={i} className="space-y-1">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-2 w-full" />
          </div>
        ))}
      </div>
    </CardContent>
  </Card>
)

export const FullPageSkeleton = () => (
  <div className="min-h-screen bg-background">
    <div className="border-b border-white/10 backdrop-blur-xl">
      <div className="container flex items-center justify-between py-4">
        <div className="flex items-center space-x-4">
          <Skeleton className="h-8 w-8" />
          <Skeleton className="h-6 w-20" />
          <Skeleton className="h-6 w-24" />
        </div>
        <div className="flex items-center gap-4">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-8 w-24" />
        </div>
      </div>
    </div>
    
    <div className="container py-8">
      <Skeleton className="h-10 w-full mb-6" />
      <PortfolioSkeleton />
    </div>
  </div>
)