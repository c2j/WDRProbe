
import React, { useState } from 'react';
import { Flame, AlertCircle, Loader2 } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { ApiService } from '../services/apiService';
import type { HeatmapData, HeatmapNode } from '../types';

const severityColor: Record<string, string> = {
  Negligible: 'bg-green-100 text-green-800 border-green-200',
  Minor: 'bg-lime-100 text-lime-800 border-lime-200',
  Moderate: 'bg-yellow-100 text-yellow-800 border-yellow-200',
  Severe: 'bg-orange-100 text-orange-800 border-orange-200',
  Extreme: 'bg-red-100 text-red-800 border-red-200',
};

const PlanHeatmap: React.FC = () => {
  const { t } = useI18n();
  const [planText, setPlanText] = useState('');
  const [data, setData] = useState<HeatmapData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!planText.trim()) return;
    setLoading(true);
    setError(null);
    setData(null);
    try {
      const result = await ApiService.getExplainHeatmap(planText);
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
        <div className="p-2 bg-gradient-to-br from-red-500 to-orange-500 rounded-lg shadow-md">
          <Flame className="text-white" size={24} />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-gray-800">{t('vis.diagnostic.heatmap')}</h1>
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
          <span className="text-amber-700 text-sm">{t('vis.diagnostic.heatmapNoData')}</span>
        </div>
      )}

      {data && (
        <>
          <div className="grid grid-cols-3 gap-4">
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
              <p className="text-xs font-medium text-gray-500 uppercase tracking-wider">{t('vis.diagnostic.maxQerror')}</p>
              <p className="text-3xl font-bold text-gray-800 mt-1">{data.summary.maxQerror.toFixed(1)}<span className="text-sm font-normal text-gray-400 ml-1">x</span></p>
            </div>
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
              <p className="text-xs font-medium text-gray-500 uppercase tracking-wider">{t('vis.diagnostic.avgQerror')}</p>
              <p className="text-3xl font-bold text-gray-800 mt-1">{data.summary.avgQerror.toFixed(1)}<span className="text-sm font-normal text-gray-400 ml-1">x</span></p>
            </div>
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
              <p className="text-xs font-medium text-gray-500 uppercase tracking-wider">{t('vis.diagnostic.nodesWithDeviation')}</p>
              <p className="text-3xl font-bold text-gray-800 mt-1">{data.summary.nodesWithDeviation}</p>
            </div>
          </div>

          <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="bg-gray-50 border-b border-gray-200">
                    <th className="text-left px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.operation')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.estimatedCost')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.actualCost')}</th>
                    <th className="text-right px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.qError')}</th>
                    <th className="text-center px-5 py-3.5 font-semibold text-gray-600">{t('vis.diagnostic.severity')}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {data.nodes.map((node: HeatmapNode, i: number) => (
                    <tr key={i} className="hover:bg-gray-50 transition-colors">
                      <td className="px-5 py-3 font-mono text-gray-700 text-xs">{node.operation}</td>
                      <td className="px-5 py-3 text-right text-gray-700 tabular-nums">{node.estimatedCost.toLocaleString()}</td>
                      <td className="px-5 py-3 text-right text-gray-700 tabular-nums">{node.actualCost.toLocaleString()}</td>
                      <td className="px-5 py-3 text-right font-medium tabular-nums">{node.qError.toFixed(1)}x</td>
                      <td className="px-5 py-3 text-center">
                        <span className={`inline-block px-2.5 py-0.5 rounded-full text-xs font-medium border ${severityColor[node.severity] || 'bg-gray-100 text-gray-700'}`}>
                          {node.severity}
                        </span>
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

export default PlanHeatmap;
