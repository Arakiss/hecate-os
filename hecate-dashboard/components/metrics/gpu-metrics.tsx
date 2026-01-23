'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { GpuIcon } from '@hugeicons/react';

interface GpuMetricsProps {
  gpus: Array<{
    index: number;
    name: string;
    temperature: number;
    power_w: number;
    memory_used_mb: number;
    memory_total_mb: number;
    utilization: number;
  }>;
}

export default function GpuMetrics({ gpus }: GpuMetricsProps) {
  const [selectedGpu, setSelectedGpu] = useState(0);

  if (!gpus || gpus.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <GpuIcon className="h-5 w-5" />
            GPU Information
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-muted-foreground py-8">
            No NVIDIA GPUs detected
          </p>
        </CardContent>
      </Card>
    );
  }

  const currentGpu = gpus[selectedGpu] || gpus[0];
  const vramUsagePercent = (currentGpu.memory_used_mb / currentGpu.memory_total_mb) * 100;
  
  const tempStatus = currentGpu.temperature > 85 ? 'destructive'
    : currentGpu.temperature > 75 ? 'secondary'
    : 'default';

  const utilizationStatus = currentGpu.utilization > 90 ? 'destructive'
    : currentGpu.utilization > 70 ? 'secondary'
    : 'default';

  if (gpus.length === 1) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <GpuIcon className="h-5 w-5" />
            {currentGpu.name}
          </CardTitle>
          <CardDescription>GPU performance and utilization metrics</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Temperature</p>
              <div className="flex items-center gap-2">
                <p className="text-2xl font-bold">{currentGpu.temperature}°C</p>
                <Badge variant={tempStatus as any} className="text-xs">
                  {tempStatus === 'destructive' ? 'Hot' : tempStatus === 'secondary' ? 'Warm' : 'OK'}
                </Badge>
              </div>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Power Draw</p>
              <p className="text-2xl font-bold">{currentGpu.power_w}W</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Utilization</p>
              <div className="flex items-center gap-2">
                <p className="text-2xl font-bold">{currentGpu.utilization}%</p>
                <Badge variant={utilizationStatus as any} className="text-xs">
                  {utilizationStatus === 'destructive' ? 'High' : utilizationStatus === 'secondary' ? 'Med' : 'Low'}
                </Badge>
              </div>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">VRAM</p>
              <p className="text-2xl font-bold">
                {(currentGpu.memory_used_mb / 1024).toFixed(1)} GB
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium">GPU Utilization</span>
                <span>{currentGpu.utilization}%</span>
              </div>
              <Progress value={currentGpu.utilization} />
            </div>

            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="font-medium">VRAM Usage</span>
                <span>
                  {currentGpu.memory_used_mb} / {currentGpu.memory_total_mb} MB
                </span>
              </div>
              <Progress value={vramUsagePercent} />
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <GpuIcon className="h-5 w-5" />
          GPU Performance ({gpus.length} GPUs)
        </CardTitle>
        <CardDescription>Multi-GPU system monitoring</CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="0" className="w-full">
          <TabsList className="grid w-full" style={{ gridTemplateColumns: `repeat(${gpus.length}, 1fr)` }}>
            {gpus.map((gpu) => (
              <TabsTrigger key={gpu.index} value={gpu.index.toString()}>
                GPU {gpu.index}
              </TabsTrigger>
            ))}
          </TabsList>
          
          {gpus.map((gpu) => {
            const memPercent = (gpu.memory_used_mb / gpu.memory_total_mb) * 100;
            const tempSt = gpu.temperature > 85 ? 'destructive'
              : gpu.temperature > 75 ? 'secondary'
              : 'default';
            const utilSt = gpu.utilization > 90 ? 'destructive'
              : gpu.utilization > 70 ? 'secondary'
              : 'default';

            return (
              <TabsContent key={gpu.index} value={gpu.index.toString()} className="space-y-6">
                <div>
                  <h4 className="font-semibold mb-4">{gpu.name}</h4>
                  
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                    <div>
                      <p className="text-sm text-muted-foreground">Temperature</p>
                      <div className="flex items-center gap-2">
                        <p className="text-2xl font-bold">{gpu.temperature}°C</p>
                        <Badge variant={tempSt as any} className="text-xs">
                          {tempSt === 'destructive' ? 'Hot' : tempSt === 'secondary' ? 'Warm' : 'OK'}
                        </Badge>
                      </div>
                    </div>
                    <div>
                      <p className="text-sm text-muted-foreground">Power Draw</p>
                      <p className="text-2xl font-bold">{gpu.power_w}W</p>
                    </div>
                    <div>
                      <p className="text-sm text-muted-foreground">Utilization</p>
                      <div className="flex items-center gap-2">
                        <p className="text-2xl font-bold">{gpu.utilization}%</p>
                        <Badge variant={utilSt as any} className="text-xs">
                          {utilSt === 'destructive' ? 'High' : utilSt === 'secondary' ? 'Med' : 'Low'}
                        </Badge>
                      </div>
                    </div>
                    <div>
                      <p className="text-sm text-muted-foreground">VRAM</p>
                      <p className="text-2xl font-bold">
                        {(gpu.memory_used_mb / 1024).toFixed(1)} GB
                      </p>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="font-medium">GPU Core Utilization</span>
                        <span>{gpu.utilization}%</span>
                      </div>
                      <Progress value={gpu.utilization} />
                    </div>

                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="font-medium">VRAM Usage</span>
                        <span>
                          {gpu.memory_used_mb} / {gpu.memory_total_mb} MB ({memPercent.toFixed(1)}%)
                        </span>
                      </div>
                      <Progress value={memPercent} />
                    </div>
                  </div>
                </div>
              </TabsContent>
            );
          })}
        </Tabs>
      </CardContent>
    </Card>
  );
}