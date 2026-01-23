'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { CpuHigh02Icon } from '@hugeicons/react';

interface CpuMetricsProps {
  metrics: {
    usage_percent: number;
    per_core: number[];
    temperature: number | null;
    frequency: number;
    load_avg: [number, number, number];
  };
  history: Array<{
    usage_percent: number;
    per_core: number[];
    temperature: number | null;
    frequency: number;
    load_avg: [number, number, number];
  }>;
  detailed?: boolean;
}

export default function CpuMetrics({ metrics, history, detailed = false }: CpuMetricsProps) {
  const tempStatus = metrics.temperature 
    ? metrics.temperature > 85 ? 'destructive' 
    : metrics.temperature > 75 ? 'secondary' 
    : 'default'
    : 'default';

  const usageStatus = metrics.usage_percent > 90 ? 'destructive'
    : metrics.usage_percent > 70 ? 'secondary'
    : 'default';

  if (!detailed) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CpuHigh02Icon className="h-5 w-5" />
            CPU Performance
          </CardTitle>
          <CardDescription>Overall usage and temperature</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Usage</span>
            <Badge variant={usageStatus as any}>{metrics.usage_percent.toFixed(1)}%</Badge>
          </div>
          <Progress value={metrics.usage_percent} />
          
          <div className="grid grid-cols-2 gap-4 pt-2">
            <div>
              <p className="text-xs text-muted-foreground">Temperature</p>
              <p className="text-lg font-semibold">
                {metrics.temperature?.toFixed(1) || '--'}°C
              </p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">Frequency</p>
              <p className="text-lg font-semibold">{metrics.frequency} MHz</p>
            </div>
          </div>

          <div className="pt-2 border-t">
            <p className="text-xs text-muted-foreground mb-1">Load Average</p>
            <div className="flex justify-between text-sm">
              <span>1m: {metrics.load_avg[0].toFixed(2)}</span>
              <span>5m: {metrics.load_avg[1].toFixed(2)}</span>
              <span>15m: {metrics.load_avg[2].toFixed(2)}</span>
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
            <CpuHigh02Icon className="h-5 w-5" />
            CPU Detailed Metrics
          </CardTitle>
          <CardDescription>Per-core usage and detailed statistics</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Overall Usage</p>
              <p className="text-2xl font-bold">{metrics.usage_percent.toFixed(1)}%</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Temperature</p>
              <div className="flex items-center gap-2">
                <p className="text-2xl font-bold">{metrics.temperature?.toFixed(1) || '--'}°C</p>
                <Badge variant={tempStatus as any} className="text-xs">
                  {tempStatus === 'destructive' ? 'Hot' : tempStatus === 'secondary' ? 'Warm' : 'Normal'}
                </Badge>
              </div>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Frequency</p>
              <p className="text-2xl font-bold">{metrics.frequency} MHz</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Cores</p>
              <p className="text-2xl font-bold">{metrics.per_core.length}</p>
            </div>
          </div>

          <div className="space-y-3">
            <h4 className="text-sm font-semibold">Per-Core Usage</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {metrics.per_core.map((usage, i) => (
                <div key={i} className="space-y-1">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Core {i}</span>
                    <span className="font-medium">{usage.toFixed(1)}%</span>
                  </div>
                  <Progress 
                    value={usage} 
                    className="h-2"
                  />
                </div>
              ))}
            </div>
          </div>

          <div className="pt-4 border-t">
            <h4 className="text-sm font-semibold mb-3">System Load</h4>
            <div className="grid grid-cols-3 gap-4">
              <div>
                <p className="text-xs text-muted-foreground">1 minute</p>
                <p className="text-lg font-semibold">{metrics.load_avg[0].toFixed(2)}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">5 minutes</p>
                <p className="text-lg font-semibold">{metrics.load_avg[1].toFixed(2)}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">15 minutes</p>
                <p className="text-lg font-semibold">{metrics.load_avg[2].toFixed(2)}</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}