'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { HardDrive01Icon } from '@hugeicons/react';

interface DiskMetricsProps {
  disks: Array<{
    name: string;
    mount_point: string;
    total_gb: number;
    used_gb: number;
    read_mb_s: number;
    write_mb_s: number;
  }>;
  detailed?: boolean;
}

export default function DiskMetrics({ disks, detailed = false }: DiskMetricsProps) {
  const getUsageStatus = (percent: number) => {
    if (percent > 95) return 'destructive';
    if (percent > 85) return 'secondary';
    return 'default';
  };

  const formatSize = (gb: number): string => {
    if (gb >= 1000) return `${(gb / 1000).toFixed(2)} TB`;
    return `${gb.toFixed(1)} GB`;
  };

  if (!detailed) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <HardDrive01Icon className="h-5 w-5" />
            Storage Usage
          </CardTitle>
          <CardDescription>Disk space utilization</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {disks.map((disk) => {
            const usagePercent = (disk.used_gb / disk.total_gb) * 100;
            const status = getUsageStatus(usagePercent);
            
            return (
              <div key={disk.mount_point} className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">{disk.mount_point}</span>
                  <Badge variant={status as any} className="text-xs">
                    {usagePercent.toFixed(1)}%
                  </Badge>
                </div>
                <Progress value={usagePercent} />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>{formatSize(disk.used_gb)} used</span>
                  <span>{formatSize(disk.total_gb)} total</span>
                </div>
              </div>
            );
          })}
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {disks.map((disk) => {
        const usagePercent = (disk.used_gb / disk.total_gb) * 100;
        const freeGb = disk.total_gb - disk.used_gb;
        const status = getUsageStatus(usagePercent);

        return (
          <Card key={disk.mount_point}>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <HardDrive01Icon className="h-4 w-4" />
                {disk.mount_point}
              </CardTitle>
              <CardDescription>{disk.name}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-xs text-muted-foreground">Total Space</p>
                  <p className="text-lg font-semibold">{formatSize(disk.total_gb)}</p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Used Space</p>
                  <div className="flex items-center gap-2">
                    <p className="text-lg font-semibold">{formatSize(disk.used_gb)}</p>
                    <Badge variant={status as any} className="text-xs">
                      {usagePercent.toFixed(0)}%
                    </Badge>
                  </div>
                </div>
              </div>

              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span>Disk Usage</span>
                  <span>{usagePercent.toFixed(1)}%</span>
                </div>
                <Progress value={usagePercent} className="h-3" />
                <div className="flex justify-between text-xs text-muted-foreground mt-1">
                  <span>Used: {formatSize(disk.used_gb)}</span>
                  <span>Free: {formatSize(freeGb)}</span>
                </div>
              </div>

              {(disk.read_mb_s > 0 || disk.write_mb_s > 0) && (
                <div className="pt-2 border-t">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <p className="text-xs text-muted-foreground">Read Speed</p>
                      <p className="font-medium">{disk.read_mb_s.toFixed(2)} MB/s</p>
                    </div>
                    <div>
                      <p className="text-xs text-muted-foreground">Write Speed</p>
                      <p className="font-medium">{disk.write_mb_s.toFixed(2)} MB/s</p>
                    </div>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}