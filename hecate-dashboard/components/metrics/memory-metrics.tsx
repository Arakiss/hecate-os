'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Memory01Icon } from '@hugeicons/react';

interface MemoryMetricsProps {
  metrics: {
    total_gb: number;
    used_gb: number;
    available_gb: number;
    swap_total_gb: number;
    swap_used_gb: number;
    cache_gb: number;
  };
  history: Array<{
    total_gb: number;
    used_gb: number;
    available_gb: number;
    swap_total_gb: number;
    swap_used_gb: number;
    cache_gb: number;
  }>;
  detailed?: boolean;
}

export default function MemoryMetrics({ metrics, history, detailed = false }: MemoryMetricsProps) {
  const memUsagePercent = (metrics.used_gb / metrics.total_gb) * 100;
  const swapUsagePercent = metrics.swap_total_gb > 0 
    ? (metrics.swap_used_gb / metrics.swap_total_gb) * 100 
    : 0;

  const memStatus = memUsagePercent > 90 ? 'destructive'
    : memUsagePercent > 80 ? 'secondary'
    : 'default';

  if (!detailed) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Memory01Icon className="h-5 w-5" />
            Memory Usage
          </CardTitle>
          <CardDescription>RAM and swap utilization</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">RAM</span>
              <Badge variant={memStatus as any}>
                {metrics.used_gb.toFixed(1)} / {metrics.total_gb.toFixed(1)} GB
              </Badge>
            </div>
            <Progress value={memUsagePercent} />
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Swap</span>
              <span className="text-sm text-muted-foreground">
                {metrics.swap_used_gb.toFixed(1)} / {metrics.swap_total_gb.toFixed(1)} GB
              </span>
            </div>
            <Progress value={swapUsagePercent} className="h-2" />
          </div>

          <div className="grid grid-cols-2 gap-4 pt-2 border-t">
            <div>
              <p className="text-xs text-muted-foreground">Available</p>
              <p className="text-lg font-semibold">{metrics.available_gb.toFixed(1)} GB</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">Cache</p>
              <p className="text-lg font-semibold">{metrics.cache_gb.toFixed(1)} GB</p>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Memory01Icon className="h-5 w-5" />
            Memory Detailed Metrics
          </CardTitle>
          <CardDescription>Comprehensive memory usage breakdown</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Total Memory</p>
              <p className="text-2xl font-bold">{metrics.total_gb.toFixed(1)} GB</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Used Memory</p>
              <div className="flex items-center gap-2">
                <p className="text-2xl font-bold">{metrics.used_gb.toFixed(1)} GB</p>
                <Badge variant={memStatus as any} className="text-xs">
                  {memUsagePercent.toFixed(0)}%
                </Badge>
              </div>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Available</p>
              <p className="text-2xl font-bold">{metrics.available_gb.toFixed(1)} GB</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium">Physical Memory</span>
                <span>{memUsagePercent.toFixed(1)}% used</span>
              </div>
              <Progress value={memUsagePercent} className="h-3" />
              <div className="grid grid-cols-3 gap-2 mt-2 text-xs text-muted-foreground">
                <div>Used: {metrics.used_gb.toFixed(1)} GB</div>
                <div>Cache: {metrics.cache_gb.toFixed(1)} GB</div>
                <div>Free: {(metrics.total_gb - metrics.used_gb).toFixed(1)} GB</div>
              </div>
            </div>

            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium">Swap Memory</span>
                <span>{swapUsagePercent.toFixed(1)}% used</span>
              </div>
              <Progress value={swapUsagePercent} className="h-3" />
              <div className="grid grid-cols-2 gap-2 mt-2 text-xs text-muted-foreground">
                <div>Used: {metrics.swap_used_gb.toFixed(1)} GB</div>
                <div>Free: {(metrics.swap_total_gb - metrics.swap_used_gb).toFixed(1)} GB</div>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t">
            <h4 className="text-sm font-semibold mb-3">Memory Distribution</h4>
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Application Memory</span>
                <span className="text-sm font-medium">
                  {(metrics.used_gb - metrics.cache_gb).toFixed(1)} GB
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Cache & Buffers</span>
                <span className="text-sm font-medium">{metrics.cache_gb.toFixed(1)} GB</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">Available</span>
                <span className="text-sm font-medium">{metrics.available_gb.toFixed(1)} GB</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}