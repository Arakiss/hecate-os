'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Wifi01Icon, ArrowDownIcon, ArrowUpIcon } from '@hugeicons/react';

interface NetworkMetricsProps {
  metrics: {
    interfaces: Array<{
      name: string;
      rx_mb_s: number;
      tx_mb_s: number;
    }>;
    total_rx_mb_s: number;
    total_tx_mb_s: number;
  };
  history: Array<{
    interfaces: Array<{
      name: string;
      rx_mb_s: number;
      tx_mb_s: number;
    }>;
    total_rx_mb_s: number;
    total_tx_mb_s: number;
  }>;
}

export default function NetworkMetrics({ metrics, history }: NetworkMetricsProps) {
  const formatSpeed = (mbps: number): string => {
    if (mbps < 0.01) return '0 MB/s';
    if (mbps < 1) return `${(mbps * 1024).toFixed(1)} KB/s`;
    return `${mbps.toFixed(2)} MB/s`;
  };

  const getSpeedColor = (speed: number): string => {
    if (speed > 100) return 'text-red-600';
    if (speed > 50) return 'text-orange-600';
    if (speed > 10) return 'text-yellow-600';
    if (speed > 1) return 'text-green-600';
    return 'text-muted-foreground';
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wifi01Icon className="h-5 w-5" />
          Network Activity
        </CardTitle>
        <CardDescription>Real-time network interface statistics</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4 p-4 bg-muted/50 rounded-lg">
          <div className="flex items-center gap-2">
            <ArrowDownIcon className="h-4 w-4 text-green-600" />
            <div>
              <p className="text-xs text-muted-foreground">Total Download</p>
              <p className={`text-lg font-bold ${getSpeedColor(metrics.total_rx_mb_s)}`}>
                {formatSpeed(metrics.total_rx_mb_s)}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <ArrowUpIcon className="h-4 w-4 text-blue-600" />
            <div>
              <p className="text-xs text-muted-foreground">Total Upload</p>
              <p className={`text-lg font-bold ${getSpeedColor(metrics.total_tx_mb_s)}`}>
                {formatSpeed(metrics.total_tx_mb_s)}
              </p>
            </div>
          </div>
        </div>

        <Separator />

        <div className="space-y-3">
          <h4 className="text-sm font-semibold">Interfaces</h4>
          {metrics.interfaces.map((iface) => {
            const isActive = iface.rx_mb_s > 0.001 || iface.tx_mb_s > 0.001;
            return (
              <div key={iface.name} className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{iface.name}</span>
                    {isActive && (
                      <Badge variant="outline" className="text-xs">Active</Badge>
                    )}
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="flex items-center gap-1">
                      <ArrowDownIcon className="h-3 w-3 text-green-600" />
                      <span className={getSpeedColor(iface.rx_mb_s)}>
                        {formatSpeed(iface.rx_mb_s)}
                      </span>
                    </span>
                    <span className="flex items-center gap-1">
                      <ArrowUpIcon className="h-3 w-3 text-blue-600" />
                      <span className={getSpeedColor(iface.tx_mb_s)}>
                        {formatSpeed(iface.tx_mb_s)}
                      </span>
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        {metrics.interfaces.length === 0 && (
          <p className="text-center text-sm text-muted-foreground py-4">
            No network interfaces detected
          </p>
        )}
      </CardContent>
    </Card>
  );
}