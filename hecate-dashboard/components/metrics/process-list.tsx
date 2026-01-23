'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Separator } from '@/components/ui/separator';
import { Activity01Icon, CpuHigh02Icon, Memory01Icon } from '@hugeicons/react';

interface ProcessListProps {
  processes: {
    total_count: number;
    running_count: number;
    top_by_cpu: Array<{
      pid: number;
      name: string;
      cpu_percent: number;
      memory_mb: number;
    }>;
    top_by_memory: Array<{
      pid: number;
      name: string;
      cpu_percent: number;
      memory_mb: number;
    }>;
  };
}

export default function ProcessList({ processes }: ProcessListProps) {
  const formatMemory = (mb: number): string => {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  };

  const getCpuBadgeVariant = (percent: number) => {
    if (percent > 50) return 'destructive';
    if (percent > 25) return 'secondary';
    return 'outline';
  };

  const getMemoryBadgeVariant = (mb: number) => {
    if (mb > 4096) return 'destructive';  // > 4GB
    if (mb > 2048) return 'secondary';     // > 2GB
    return 'outline';
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity01Icon className="h-5 w-5" />
            Process Overview
          </CardTitle>
          <CardDescription>System process statistics</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <div className="p-4 bg-muted/50 rounded-lg">
              <p className="text-sm text-muted-foreground">Total Processes</p>
              <p className="text-2xl font-bold">{processes.total_count}</p>
            </div>
            <div className="p-4 bg-muted/50 rounded-lg">
              <p className="text-sm text-muted-foreground">Running</p>
              <p className="text-2xl font-bold text-green-600">{processes.running_count}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Top Processes</CardTitle>
          <CardDescription>Resource-intensive processes</CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="cpu" className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="cpu" className="flex items-center gap-2">
                <CpuHigh02Icon className="h-4 w-4" />
                By CPU
              </TabsTrigger>
              <TabsTrigger value="memory" className="flex items-center gap-2">
                <Memory01Icon className="h-4 w-4" />
                By Memory
              </TabsTrigger>
            </TabsList>

            <TabsContent value="cpu" className="space-y-2 mt-4">
              {processes.top_by_cpu.map((proc, index) => (
                <div key={proc.pid}>
                  <div className="flex items-center justify-between py-2">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-mono text-muted-foreground">
                          #{proc.pid}
                        </span>
                        <span className="text-sm font-medium truncate">
                          {proc.name}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant={getCpuBadgeVariant(proc.cpu_percent) as any}>
                        {proc.cpu_percent.toFixed(1)}% CPU
                      </Badge>
                      <span className="text-xs text-muted-foreground">
                        {formatMemory(proc.memory_mb)}
                      </span>
                    </div>
                  </div>
                  {index < processes.top_by_cpu.length - 1 && <Separator />}
                </div>
              ))}
            </TabsContent>

            <TabsContent value="memory" className="space-y-2 mt-4">
              {processes.top_by_memory.map((proc, index) => (
                <div key={proc.pid}>
                  <div className="flex items-center justify-between py-2">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-mono text-muted-foreground">
                          #{proc.pid}
                        </span>
                        <span className="text-sm font-medium truncate">
                          {proc.name}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant={getMemoryBadgeVariant(proc.memory_mb) as any}>
                        {formatMemory(proc.memory_mb)}
                      </Badge>
                      <span className="text-xs text-muted-foreground">
                        {proc.cpu_percent.toFixed(1)}% CPU
                      </span>
                    </div>
                  </div>
                  {index < processes.top_by_memory.length - 1 && <Separator />}
                </div>
              ))}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
}