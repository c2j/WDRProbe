
import React, { useState } from 'react';
import { Activity, AlertCircle, Loader2, Cpu } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { ApiService } from '../services/apiService';
import type { WaterfallData, WaterfallNode } from '../types';

const PlanWaterfall: React.FC = () => {
  const { t } = useI18n();
  const [planText, setPlanText] = useState('');
  const [data, setData] = useState<WaterfallData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!planText.trim()) return;
    setLoading(true);
    setError(null);
    setData(null);
    try {
      const result = await ApiService.getExplainWaterfall(planText);
      setData(result);
    } catch (e: any) {
      setError(e.message || t('vis.diagnostic.error', { msg: 'Unknown error' }));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center space-x-3">
        <div className="p-2 bg-gradient-to-br from-cyan-500 to-blue-500 rounded-lg shadow-md">
          <Activity className="text-white" size={24} />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-gray-800">{t('vis.diagnostic.waterfall')}</h1>
          <p className="text-sm text-gray-500 mt-1">{t('vis.diagnostic.title')}</p>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          {t('vis.diagnostic.planInput')}
        </label>
        <textarea
          value={planText}
          onChange={(e) => setPlanText(e.target.value)}
          placeholder={t('vis.diagnostic.planPlaceholder')}
          rows={6}
          className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm font-mono bg-gray-50 resize-y"
        />
        <button
          onClick={handleAnalyze}
          disabled={loading || !planText.trim()}
          className="mt-4 px-6 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium text-sm flex items-center space-x-2"
        >
          {loading && <Loader2 size={16} className="animate-spin" />}
          <span>{t('vis.diagnostic.analyze')}</span>
        </button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-xl p-4 flex items-start space-x-3">
          <AlertCircle className="text-red-500 shrink-0 mt-0.5" size={18} />
          <span className="text-red-700 text-sm">{error}</span>
        </div>
      )}

      {data === null && !loading && !error && planText.trim() && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-start space-x-3">
          <AlertCircle className="text-amber-500 shrink-0 mt-0.5" size={18} />
          <span className="text-amber-700 text-sm">{t('vis.diagnostic.waterfallNoData')}</span>
        </div>
      )}

      {data && (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
              <div className="flex items-center space-x-2 mb-3">
                <Cpu size={18} className="text-red-500" />
                <h3 className="font-semibold text-gray-700">{t('vis.diagnostic.cpuBottlenecks')}</h3>
              </div>
              {data.bottlenecks.cpuBottlenecks.length === 0 ? (
                <p className="text-sm text-gray-400">{t('vis.diagnostic.noBottlenecks')}</p>
              ) : (
                <div className="flex flex-wrap gap-2">
                  {data.bottlenecks.cpuBottlenecks.map((name, i) => (
                    <span key={i} className="inline-block px-3 py-1 bg-red-50 text-red-700 border border-red-200 rounded-full text-xs font-medium">
                      {name}
                    </span>
                  ))}
                </div>
              )}
            </div>
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
              <div className="flex items-center space-x-2 mb-3">
                <Activity size={18} className="text-purple-500" />
                <h3 className="font-semibold text-gray-700">{t('vis.diagnostic.memoryBottlenecks')}</h3>
              </div>
              {data.bottlenecks.memoryBottlenecks.length === 0 ? (
                <p className="text-sm text-gray-400">{t('vis.diagnostic.noBottlenecks')}</p>
              ) : (
                <div className="flex flex-wrap gap-2">
                  {data.bottlenecks.memoryBottlenecks.map((name, i) => (
                    <span key={i} className="inline-block px-3 py-1 bg-purple-50 text-purple-700 border border-purple-200 rounded-full text-xs font-medium">
                      {name}
                    </span>
                  ))}
                </div>
              )}
            </div>
          </div>

          <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="bg-gray-50 border-b border-gray-200">
                    <th className="text-left px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.operation')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.cpuTime')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.memory')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.percentage')}</th>
                    <th className="px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.percentage')}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {data.nodes.map((node: WaterfallNode, i: number) => (
                    <tr key={i} className="hover:bg-gray-50 transition-colors">
                      <td className="px-5 py-3 font-mono text-gray-700 text-xs max-w-[240px] truncate">{node.operation}</td>
                      <td className="px-5 py-3 text-right text-gray-700 tabular-nums">{node.cpuTime.toLocaleString()}</td>
                      <td className="px-5 py-3 text-right text-gray-700 tabular-nums">{node.memoryKb.toLocaleString()}</td>
                      <td className="px-5 py-3 text-right font-medium tabular-nums">{node.percentage.toFixed(1)}%</td>
                      <td className="px-5 py-3">
                        <div className="flex items-center space-x-2">
                          <div className="flex-1 h-4 bg-gray-100 rounded-full overflow-hidden">
                            <div
                              className="h-full bg-gradient-to-r from-cyan-400 to-blue-500 rounded-full transition-all"
                              style={{ width: `${Math.min(node.percentage, 100)}%` }}
                            />
                          </div>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default PlanWaterfall;
