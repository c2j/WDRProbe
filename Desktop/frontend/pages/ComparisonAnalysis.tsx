import React, { useState, useEffect } from 'react';
import { X, ArrowUp, ArrowDown, Loader2, Sparkles, TrendingDown, TrendingUp, Minus, BarChart3, Check, Plus, Eye } from 'lucide-react';
import {
    BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid
} from 'recharts';
import { useI18n } from '../context/I18nContext';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { ApiService } from '../services/apiService';
import {
    ComparisonCategory,
    SqlComparisonMetric,
    WaitEventComparison,
    ObjectStatComparison,
    SystemMetricComparison,
    ComparisonSummary,
    WdrReport,
    WdrComparison
} from '../types';

const ComparisonAnalysis: React.FC = () => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const comparisonIdParam = searchParams.get('comparisonId');

  // Comparisons list state
  const [comparisons, setComparisons] = useState<WdrComparison[]>([]);
  const [comparisonsLoading, setComparisonsLoading] = useState(true);

  // Selected comparison for detail view
  const [selectedComparisonId, setSelectedComparisonId] = useState<number | null>(null);

  // Detail view state
  const [activeTab, setActiveTab] = useState<ComparisonCategory>('sql');
  const [loading, setLoading] = useState(false);
  const [summary, setSummary] = useState<ComparisonSummary | null>(null);
  const [metrics, setMetrics] = useState<any[]>([]);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetricComparison[]>([]);

  // New comparison dialog state
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [availableReports, setAvailableReports] = useState<WdrReport[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<number | null>(null);
  const [selectedTargetId, setSelectedTargetId] = useState<number | null>(null);
  const [creatingComparison, setCreatingComparison] = useState(false);

  const TABS: { id: ComparisonCategory, label: string }[] = [
    { id: 'sql', label: t('comp.tab.sql') },
    { id: 'wait', label: t('comp.tab.wait') },
    { id: 'obj', label: t('comp.tab.obj') },
    { id: 'sys', label: t('comp.tab.sys') }
  ];

  // Load comparisons list
  useEffect(() => {
    const loadComparisons = async () => {
      setComparisonsLoading(true);
      try {
        const data = await ApiService.getComparisons();
        const compList = Array.isArray(data) ? data : [];
        setComparisons(compList);

        // Auto-select first comparison if none selected
        if (compList.length > 0 && !selectedComparisonId && !comparisonIdParam) {
          setSelectedComparisonId(compList[0].id);
        }
      } catch (error) {
        console.error("Failed to load comparisons", error);
        setComparisons([]);
      } finally {
        setComparisonsLoading(false);
      }
    };
    loadComparisons();
  }, []);

  // Load available reports for comparison creation
  useEffect(() => {
    const loadReports = async () => {
      const reports = await ApiService.getWdrReports();
      setAvailableReports(Array.isArray(reports) ? reports : []);
    };
    loadReports();
  }, []);

  // Watch URL parameter changes
  useEffect(() => {
    if (comparisonIdParam) {
      const newComparisonId = parseInt(comparisonIdParam);
      if (newComparisonId !== selectedComparisonId) {
        setSelectedComparisonId(newComparisonId);
      }
    }
  }, [comparisonIdParam]);

  // Load comparison summary and system metrics when selection changes
  useEffect(() => {
    const fetchSummary = async () => {
        if (!selectedComparisonId) return;
        try {
            const data = await ApiService.getComparisonSummary(selectedComparisonId);
            setSummary(data);
            // Pre-fetch system metrics for the overview chart
            const sysData = await ApiService.getComparisonDetails(selectedComparisonId, 'sys');
            setSystemMetrics(sysData as SystemMetricComparison[]);
        } catch (error) {
            console.error("Failed to fetch summary", error);
            setSummary(null);
            setSystemMetrics([]);
        }
    };
    fetchSummary();
  }, [selectedComparisonId]);

  // Load comparison details for active tab
  useEffect(() => {
    const fetchData = async () => {
        if (!selectedComparisonId) return;
        setLoading(true);
        try {
            const data = await ApiService.getComparisonDetails(selectedComparisonId, activeTab);
            // Ensure data is always an array
            if (Array.isArray(data)) {
                setMetrics(data);
            } else {
                console.warn('Comparison details is not an array:', data);
                setMetrics([]);
            }
        } catch (error) {
            console.error("Failed to fetch comparison details", error);
            setMetrics([]);
        } finally {
            setLoading(false);
        }
    };

    fetchData();
  }, [activeTab, selectedComparisonId]);

  const handleCreateComparison = async () => {
    if (!selectedSourceId || !selectedTargetId || selectedSourceId === selectedTargetId) {
      alert('请选择两个不同的报告');
      return;
    }

    setCreatingComparison(true);
    try {
      const result = await ApiService.createComparison({
        report1Id: selectedSourceId,
        report2Id: selectedTargetId,
        customName: `Comparison: Report ${selectedSourceId} vs ${selectedTargetId}`
      });
      // Reload comparisons list and select the new one
      const updatedComparisons = await ApiService.getComparisons();
      setComparisons(Array.isArray(updatedComparisons) ? updatedComparisons : []);
      setSelectedComparisonId(result.id);
      setShowCreateDialog(false);
      // Reset form
      setSelectedSourceId(null);
      setSelectedTargetId(null);
      navigate(`/comparison?comparisonId=${result.id}`, { replace: true });
    } catch (error) {
      console.error('Failed to create comparison:', error);
      alert('创建对比失败，请重试');
    } finally {
      setCreatingComparison(false);
    }
  };

  const handleSelectComparison = (compId: number) => {
    setSelectedComparisonId(compId);
    navigate(`/comparison?comparisonId=${compId}`, { replace: true });
  };

  const renderChange = (val: number) => {
      const colorClass = val > 0 ? 'text-red-500' : val < 0 ? 'text-green-500' : 'text-gray-500';
      return (
          <span className={`flex items-center justify-end ${colorClass}`}>
              {val > 0 ? '+' : ''}{Math.abs(val)}%
              {val > 0 ? <ArrowUp size={14} className="ml-1"/> : val < 0 ? <ArrowDown size={14} className="ml-1"/> : null}
          </span>
      );
  };

  // ========== Comparisons List Section ==========
  const renderComparisonsList = () => {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden">
        {/* List Header */}
        <div className="flex justify-between items-center p-4 border-b border-gray-200">
          <div>
            <h2 className="text-lg font-semibold text-gray-800">{t('comp.title') || '对比分析'}</h2>
            <p className="text-sm text-gray-500 mt-1">选择一个对比查看详细分析</p>
          </div>
          <button
            onClick={() => setShowCreateDialog(true)}
            className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700 transition-colors font-medium"
          >
            <Plus size={16} className="mr-2" />
            {t('comp.new') || '新建对比'}
          </button>
        </div>

        {/* Comparisons Table */}
        <table className="w-full text-left text-sm">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-3 font-medium text-gray-500 w-20">ID</th>
              <th className="px-4 py-3 font-medium text-gray-500">{t('comp.name') || '名称'}</th>
              <th className="px-4 py-3 font-medium text-gray-500">{t('comp.reports') || '报告'}</th>
              <th className="px-4 py-3 font-medium text-gray-500">{t('comp.createdAt') || '创建时间'}</th>
              <th className="px-4 py-3 font-medium text-gray-500 w-20">{t('comp.actions') || '操作'}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {comparisonsLoading ? (
              <tr><td colSpan={5} className="p-6 text-center text-gray-500">加载中...</td></tr>
            ) : comparisons.length === 0 ? (
              <tr><td colSpan={5} className="p-6 text-center text-gray-500">暂无对比分析记录</td></tr>
            ) : (
              comparisons.map(comp => {
                const isSelected = comp.id === selectedComparisonId;
                return (
                  <tr
                    key={comp.id}
                    className={`cursor-pointer transition-colors ${isSelected ? 'bg-blue-50 border-l-4 border-blue-600' : 'hover:bg-gray-50'}`}
                    onClick={() => handleSelectComparison(comp.id)}
                  >
                    <td className="px-4 py-3 text-gray-900 font-medium">#{comp.id}</td>
                    <td className="px-4 py-3 text-gray-700 font-medium">{comp.name}</td>
                    <td className="px-4 py-3">
                      <div className="flex space-x-1">
                        {comp.reportIds.map((rid, idx) => (
                          <span key={rid} className="px-2 py-1 bg-blue-50 text-blue-700 rounded text-xs font-medium">
                            #{rid}{idx < comp.reportIds.length - 1 && <span className="mx-1">vs</span>}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="px-4 py-3 text-gray-500">{comp.createdAt}</td>
                    <td className="px-4 py-3">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSelectComparison(comp.id);
                        }}
                        className="text-blue-600 hover:text-blue-800 transition-colors"
                        title="查看详情"
                      >
                        <Eye size={18} />
                      </button>
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    );
  };

  // ========== Detail View Section ==========
  const renderDetailSection = () => {
    if (!selectedComparisonId) {
      return (
        <div className="bg-white rounded-lg shadow-sm border border-gray-100 p-12">
          <div className="flex flex-col items-center justify-center text-gray-500">
            <BarChart3 size={64} className="mb-4 text-gray-300" />
            <p className="text-lg font-medium mb-2">请选择一个对比分析</p>
            <p className="text-sm">从上方列表中选择一个已创建的对比，或创建新的对比</p>
          </div>
        </div>
      );
    }

    return (
      <div className="space-y-4">
        {/* Analysis Summary */}
        {renderSummary()}

        {/* Overview Charts */}
        {renderOverviewSection()}

        {/* Tabs */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-100">
           <div className="flex border-b border-gray-200">
              {TABS.map((tab) => (
                  <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`px-6 py-4 text-sm font-medium border-b-2 transition-colors ${
                          activeTab === tab.id
                          ? 'border-blue-600 text-blue-600'
                          : 'border-transparent text-gray-500 hover:text-gray-700'
                      }`}
                  >
                      {tab.label}
                  </button>
              ))}
           </div>

           <div className="p-6">
              {renderTableContent()}
           </div>
        </div>
      </div>
    );
  };

  const renderTableContent = () => {
      if (loading) {
          return (
              <div className="flex justify-center items-center h-64 text-gray-400">
                  <Loader2 size={32} className="animate-spin mr-2" /> 加载中...
              </div>
          );
      }

      if (metrics.length === 0) {
          return (
              <div className="flex flex-col items-center justify-center h-64 text-gray-500">
                  <BarChart3 size={48} className="mb-4 text-gray-300" />
                  <p className="text-lg font-medium mb-2">暂无对比数据</p>
              </div>
          );
      }

      switch (activeTab) {
        case 'sql':
            return (
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm whitespace-nowrap">
                        <thead>
                            <tr className="bg-gray-50 border-b border-gray-100 text-xs uppercase tracking-wider text-gray-500">
                                <th className="px-4 py-3 font-semibold sticky left-0 bg-gray-50">{t('comp.fingerprint')}</th>
                                <th className="px-4 py-3 font-semibold text-right bg-blue-50/30">{t('comp.r1')} (ms)</th>
                                <th className="px-4 py-3 font-semibold text-right bg-blue-50/30">{t('comp.r2')} (ms)</th>
                                <th className="px-4 py-3 font-semibold text-right">{t('comp.change')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-100/50">{t('comp.col.cpu')} (1/2)</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-100/50">{t('comp.col.io')} (1/2)</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-50/50">{t('comp.col.phyRd')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-50/50">{t('comp.col.logRd')}</th>
                                <th className="px-4 py-3 font-semibold text-center">{t('comp.action')}</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-100">
                            {(metrics as SqlComparisonMetric[]).map((m) => (
                                <tr key={m.id} className="hover:bg-gray-50">
                                    <td className="px-4 py-3 font-mono text-xs text-gray-700 truncate max-w-xs sticky left-0 bg-white" title={m.name}>{m.name}</td>
                                    <td className="px-4 py-3 text-right bg-blue-50/10 font-medium">{m.value1}</td>
                                    <td className="px-4 py-3 text-right bg-blue-50/10 font-medium">{m.value2}</td>
                                    <td className="px-4 py-3 text-right">{renderChange(m.changeRate)}</td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-500 bg-gray-50/30 font-mono">
                                        {m.cpuTime1} / {m.cpuTime2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-500 bg-gray-50/30 font-mono">
                                        {m.ioTime1} / {m.ioTime2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-50/30">
                                        {m.physicalReads1} <span className="text-gray-300">/</span> {m.physicalReads2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-50/30">
                                        {m.logicalReads1} <span className="text-gray-300">/</span> {m.logicalReads2}
                                    </td>
                                    <td className="px-4 py-3 text-center">
                                        <button className="text-blue-600 hover:underline text-xs font-medium">{t('comp.plan')}</button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            );
        case 'wait':
            return (
                <table className="w-full text-left text-sm">
                    <thead>
                        <tr className="bg-gray-50 border-b border-gray-100">
                            <th className="px-4 py-3 font-medium text-gray-600">{t('comp.col.event')}</th>
                            <th className="px-4 py-3 font-medium text-gray-600">{t('comp.col.class')}</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.r1')} (ms)</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.r2')} (ms)</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.change')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-100">
                        {(metrics as WaitEventComparison[]).map((m) => (
                            <tr key={m.id} className="hover:bg-gray-50">
                                <td className="px-4 py-3 font-medium text-gray-700">{m.name}</td>
                                <td className="px-4 py-3 text-gray-500 text-xs">{m.waitClass}</td>
                                <td className="px-4 py-3 text-right">{m.value1}</td>
                                <td className="px-4 py-3 text-right">{m.value2}</td>
                                <td className="px-4 py-3 text-right">{renderChange(m.changeRate)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            );
        case 'obj':
            return (
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm whitespace-nowrap">
                        <thead>
                            <tr className="bg-gray-50 border-b border-gray-100 text-xs uppercase tracking-wider text-gray-500">
                                <th className="px-4 py-3 font-semibold sticky left-0 bg-gray-50">{t('comp.col.object')}</th>
                                <th className="px-4 py-3 font-semibold">{t('comp.col.schema')}</th>
                                <th className="px-4 py-3 font-semibold text-right">{t('comp.r1')} (Scans)</th>
                                <th className="px-4 py-3 font-semibold text-right">{t('comp.r2')} (Scans)</th>
                                <th className="px-4 py-3 font-semibold text-right">{t('comp.col.diff')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-50/50">{t('comp.col.heapRd')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-50/50">{t('comp.col.heapHit')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-100/50">{t('comp.col.idxRd')}</th>
                                <th className="px-4 py-3 font-semibold text-right text-gray-600 bg-gray-100/50">{t('comp.col.idxHit')}</th>
                                <th className="px-4 py-3 font-semibold text-center text-gray-600">{t('comp.col.tup')} (1/2)</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-100">
                            {(metrics as ObjectStatComparison[]).map((m) => (
                                <tr key={m.id} className="hover:bg-gray-50">
                                    <td className="px-4 py-3 font-medium text-gray-700 sticky left-0 bg-white">
                                        {m.name} <span className="text-gray-400 text-xs ml-2">({m.scanType})</span>
                                    </td>
                                    <td className="px-4 py-3 text-gray-500 text-xs">{m.schema}</td>
                                    <td className="px-4 py-3 text-right">{m.value1}</td>
                                    <td className="px-4 py-3 text-right">{m.value2}</td>
                                    <td className="px-4 py-3 text-right font-mono text-gray-600">
                                        {m.diff > 0 ? '+' : ''}{m.diff}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-50/30">
                                        {m.heapBlksRead1} / {m.heapBlksRead2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-50/30">
                                        {m.heapBlksHit1} / {m.heapBlksHit2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-100/30">
                                        {m.idxBlksRead1} / {m.idxBlksRead2}
                                    </td>
                                    <td className="px-4 py-3 text-right text-xs text-gray-600 bg-gray-100/30">
                                        {m.idxBlksHit1} / {m.idxBlksHit2}
                                    </td>
                                    <td className="px-4 py-3 text-center text-xs text-gray-500" title="Insert / Update / Delete">
                                        {m.tupleIns1}/{m.tupleUpd1}/{m.tupleDel1} <span className="text-gray-300 mx-1">vs</span> {m.tupleIns2}/{m.tupleUpd2}/{m.tupleDel2}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            );
        case 'sys':
            return (
                <table className="w-full text-left text-sm">
                    <thead>
                        <tr className="bg-gray-50 border-b border-gray-100">
                            <th className="px-4 py-3 font-medium text-gray-600">{t('comp.col.metric')}</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.r1')}</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.r2')}</th>
                            <th className="px-4 py-3 font-medium text-gray-600 text-right">{t('comp.change')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-100">
                        {(metrics as SystemMetricComparison[]).map((m) => (
                            <tr key={m.id} className="hover:bg-gray-50">
                                <td className="px-4 py-3 font-medium text-gray-700">{m.name}</td>
                                <td className="px-4 py-3 text-right">{m.value1} <span className="text-xs text-gray-400">{m.unit}</span></td>
                                <td className="px-4 py-3 text-right">{m.value2} <span className="text-xs text-gray-400">{m.unit}</span></td>
                                <td className="px-4 py-3 text-right">{renderChange(m.changeRate)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            );
        default:
            return null;
      }
  };

  const renderSummary = () => {
    if (!summary) return null;

    const status = summary.status || 'Unknown';
    const scoreChange = summary.scoreChange ?? 0;
    const conclusion = summary.conclusion || 'No conclusion available';
    const keyFindings = summary.keyFindings || [];

    const isDegraded = status === 'Degraded';
    const isImproved = status === 'Improved';

    const statusColor = isDegraded ? 'text-red-600' : isImproved ? 'text-green-600' : 'text-blue-600';
    const statusBg = isDegraded ? 'bg-red-50' : isImproved ? 'bg-green-50' : 'bg-blue-50';
    const statusBorder = isDegraded ? 'border-red-100' : isImproved ? 'border-green-100' : 'border-blue-100';
    const StatusIcon = isDegraded ? TrendingDown : isImproved ? TrendingUp : Minus;

    return (
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
            <h3 className="font-medium text-gray-800 mb-4 flex items-center">
                <Sparkles className="mr-2 text-purple-500" size={18} /> {t('comp.summary.title') || '分析摘要'}
            </h3>
            <div className="flex flex-col md:flex-row gap-6">
                <div className={`md:w-1/4 rounded-lg p-6 flex flex-col items-center justify-center border ${statusBg} ${statusBorder}`}>
                    <div className="flex items-center space-x-2 mb-2">
                        <StatusIcon className={statusColor} size={24} />
                        <span className={`text-lg font-bold ${statusColor}`}>
                            {t(`comp.status.${status.toLowerCase()}`)}
                        </span>
                    </div>
                    <span className={`text-4xl font-bold my-1 ${statusColor}`}>
                        {scoreChange > 0 ? '+' : ''}{scoreChange}%
                    </span>
                    <span className="text-xs text-gray-500 uppercase tracking-wider font-semibold opacity-75">{t('comp.summary.score') || '评分变化'}</span>
                </div>

                <div className="md:w-3/4 flex flex-col justify-between">
                    <div className="mb-4">
                        <h4 className="text-sm font-semibold text-gray-700 mb-2 uppercase tracking-wide flex items-center">
                            {t('comp.summary.conclusion') || '结论'}
                        </h4>
                        <p className="text-sm text-gray-600 leading-relaxed border-l-4 border-gray-200 pl-3">
                            {conclusion}
                        </p>
                    </div>
                    {keyFindings.length > 0 && (
                        <div>
                            <h4 className="text-sm font-semibold text-gray-700 mb-2 uppercase tracking-wide">
                                {t('comp.summary.findings') || '关键发现'}
                            </h4>
                            <ul className="list-disc list-inside text-sm text-gray-600 space-y-1.5 marker:text-gray-300">
                                {keyFindings.map((finding, idx) => (
                                    <li key={idx} className="pl-1">{finding}</li>
                                ))}
                            </ul>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
  };

  const getChartData = (metricName: string) => {
      const m = systemMetrics.find(s => s.name === metricName);
      if (!m) return [];
      return [
          { name: t('comp.r1'), value: m.value1 },
          { name: t('comp.r2'), value: m.value2 },
      ];
  };

  const renderOverviewSection = () => {
      if (systemMetrics.length === 0) return null;

      return (
          <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-100">
              <h3 className="font-medium text-gray-800 mb-4 flex items-center border-b pb-2">
                  <BarChart3 className="mr-2 text-blue-500" size={18} /> {t('comp.overview.title') || '概览'}
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 h-48">
                  <div className="flex flex-col">
                      <h4 className="text-xs font-semibold text-gray-500 text-center mb-2">{t('comp.chart.dbTime') || 'DB时间'}</h4>
                      <ResponsiveContainer width="100%" height="100%">
                          <BarChart data={getChartData('DB Time')} barCategoryGap="30%">
                              <CartesianGrid strokeDasharray="3 3" vertical={false} />
                              <XAxis dataKey="name" tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <YAxis tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <Tooltip cursor={{fill: 'transparent'}} />
                              <Bar dataKey="value" fill="#3b82f6" radius={[4, 4, 0, 0]} />
                          </BarChart>
                      </ResponsiveContainer>
                  </div>
                  <div className="flex flex-col">
                      <h4 className="text-xs font-semibold text-gray-500 text-center mb-2">{t('comp.chart.cpu') || 'CPU'}</h4>
                      <ResponsiveContainer width="100%" height="100%">
                          <BarChart data={getChartData('Average CPU Usage')} barCategoryGap="30%">
                              <CartesianGrid strokeDasharray="3 3" vertical={false} />
                              <XAxis dataKey="name" tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <YAxis tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <Tooltip cursor={{fill: 'transparent'}} />
                              <Bar dataKey="value" fill="#8b5cf6" radius={[4, 4, 0, 0]} />
                          </BarChart>
                      </ResponsiveContainer>
                  </div>
                  <div className="flex flex-col">
                      <h4 className="text-xs font-semibold text-gray-500 text-center mb-2">{t('comp.chart.io') || 'I/O'}</h4>
                      <ResponsiveContainer width="100%" height="100%">
                          <BarChart data={getChartData('IOPS')} barCategoryGap="30%">
                              <CartesianGrid strokeDasharray="3 3" vertical={false} />
                              <XAxis dataKey="name" tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <YAxis tick={{fontSize: 10}} axisLine={false} tickLine={false} />
                              <Tooltip cursor={{fill: 'transparent'}} />
                              <Bar dataKey="value" fill="#10b981" radius={[4, 4, 0, 0]} />
                          </BarChart>
                      </ResponsiveContainer>
                  </div>
              </div>
          </div>
      );
  };

  // ========== Create Comparison Dialog ==========
  const renderCreateDialog = () => {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
        <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl overflow-hidden">
          <div className="p-6 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold text-gray-900">{t('comp.new') || '新建对比'}</h2>
              <button
                onClick={() => setShowCreateDialog(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X size={20} />
              </button>
            </div>
            <p className="text-sm text-gray-500 mt-1">选择两个WDR报告进行对比</p>
          </div>

          <div className="p-6 space-y-6">
            {/* Source Report Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                源报告（基准）
              </label>
              <select
                value={selectedSourceId || ''}
                onChange={(e) => setSelectedSourceId(e.target.value ? parseInt(e.target.value) : null)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:outline-none"
              >
                <option value="">请选择报告...</option>
                {availableReports.map(report => (
                  <option key={report.id} value={report.id}>
                    #{report.id} - {report.instanceName} ({report.period})
                  </option>
                ))}
              </select>
            </div>

            {/* Target Report Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                目标报告（对比）
              </label>
              <select
                value={selectedTargetId || ''}
                onChange={(e) => setSelectedTargetId(e.target.value ? parseInt(e.target.value) : null)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:outline-none"
              >
                <option value="">请选择报告...</option>
                {availableReports
                  .filter(r => r.id !== selectedSourceId)
                  .map(report => (
                    <option key={report.id} value={report.id}>
                      #{report.id} - {report.instanceName} ({report.period})
                    </option>
                  ))}
              </select>
            </div>

            {/* Validation Message */}
            {selectedSourceId && selectedTargetId && selectedSourceId === selectedTargetId && (
              <p className="text-sm text-red-500">请选择两个不同的报告</p>
            )}
          </div>

          <div className="p-6 border-t border-gray-200 bg-gray-50 flex justify-end space-x-3">
            <button
              onClick={() => setShowCreateDialog(false)}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
            >
              {t('rep.cancel') || '取消'}
            </button>
            <button
              onClick={handleCreateComparison}
              disabled={!selectedSourceId || !selectedTargetId || selectedSourceId === selectedTargetId || creatingComparison}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
            >
              {creatingComparison ? (
                <>
                  <Loader2 size={16} className="mr-2 animate-spin" />
                  创建中...
                </>
              ) : (
                <>
                  <Check size={16} className="mr-2" />
                  创建对比
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="space-y-4">
      {/* Comparisons List */}
      {renderComparisonsList()}

      {/* Detail Section */}
      {renderDetailSection()}

      {/* Create Comparison Dialog */}
      {showCreateDialog && renderCreateDialog()}
    </div>
  );
};

export default ComparisonAnalysis;
