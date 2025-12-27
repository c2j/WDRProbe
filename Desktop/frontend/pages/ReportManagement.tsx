import React, { useEffect, useState } from 'react';
import { ApiService } from '../services/apiService';
import { WdrReport } from '../types';
import { RefreshCw, Trash2, GitCompare, Eye, Search, AlertTriangle, Plus, Check, X } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { useNavigate } from 'react-router-dom';
import UploadDialog from '../components/UploadDialog';

const ReportManagement: React.FC = () => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [reports, setReports] = useState<WdrReport[]>([]);
  const [loading, setLoading] = useState(true);
  const [showUploadDialog, setShowUploadDialog] = useState(false);

  // Selection state
  const [selectedReportIds, setSelectedReportIds] = useState<Set<number>>(new Set());
  const [allSelected, setAllSelected] = useState(false);

  // Delete Modal State
  const [reportToDelete, setReportToDelete] = useState<number | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    const data = await ApiService.getWdrReports();
    setReports(Array.isArray(data) ? data : []);
    setLoading(false);
  };

  const handleUpload = async (data: { filePath: string; instanceName: string; description: string }) => {
    await ApiService.importWdrReport(data.filePath, data.instanceName, data.description);
    loadData();
  };

  const handleDelete = async () => {
      if (reportToDelete) {
          await ApiService.deleteWdrReport(reportToDelete);
          setReportToDelete(null);
          loadData();
      }
  };

  // Checkbox handlers
  const handleSelectAll = () => {
    if (allSelected) {
      setSelectedReportIds(new Set());
    } else {
      setSelectedReportIds(new Set(reports.map(r => r.id)));
    }
    setAllSelected(!allSelected);
  };

  const handleSelectRow = (id: number) => {
    const newSelected = new Set(selectedReportIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedReportIds(newSelected);
    setAllSelected(newSelected.size === reports.length && reports.length > 0);
  };

  // Check if selected reports can be compared
  const getCompareValidation = () => {
    if (selectedReportIds.size !== 2) {
      return { valid: false, message: '请选择2个报告进行对比' };
    }

    const selectedReports = reports.filter(r => selectedReportIds.has(r.id));
    const instanceNames = new Set(selectedReports.map(r => r.instanceName));

    if (instanceNames.size > 1) {
      return { valid: false, message: '请选择同一数据库实例的报告' };
    }

    return { valid: true, reports: selectedReports };
  };

  const handleBatchCompare = async () => {
    const validation = getCompareValidation();
    if (!validation.valid || !validation.reports) {
      alert(validation.message);
      return;
    }

    const [report1, report2] = validation.reports;
    try {
      const result = await ApiService.createComparison({
        report1Id: report1.id,
        report2Id: report2.id,
        customName: `${report1.instanceName} - ${report1.period} vs ${report2.period}`
      });
      navigate(`/comparison?comparisonId=${result.id}`);
    } catch (error) {
      console.error('Failed to create comparison:', error);
      alert('创建对比失败');
    }
  };

  const compareValidation = getCompareValidation();
  const showFloatingBar = selectedReportIds.size > 0;

  return (
    <div className="space-y-4">
      {/* Header Actions */}
      <div className="flex flex-col sm:flex-row justify-between items-center bg-white p-4 rounded-lg shadow-sm border border-gray-100">
        <div className="flex space-x-3 w-full sm:w-auto mb-3 sm:mb-0">
            <div className="relative">
                <input
                    type="text"
                    placeholder={t('rep.search')}
                    className="pl-9 pr-4 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                />
                <Search size={16} className="absolute left-3 top-2.5 text-gray-400" />
            </div>
            <button className="p-2 text-gray-600 hover:bg-gray-100 rounded-md" onClick={loadData}>
                <RefreshCw size={18} />
            </button>
        </div>
        <div className="flex space-x-2">
            <button
                onClick={() => setShowUploadDialog(true)}
                className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700 transition-colors"
            >
                <Plus size={16} className="mr-2" />
                {t('rep.importReport')}
            </button>
        </div>
      </div>

      {/* Floating Action Bar */}
      {showFloatingBar && (
        <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-40 bg-gray-900 text-white px-6 py-3 rounded-full shadow-2xl flex items-center space-x-4 animate-in slide-in-from-bottom-4 fade-in duration-300">
          <span className="text-sm">
            已选择 <strong>{selectedReportIds.size}</strong> 个报告
          </span>
          {compareValidation.valid ? (
            <button
              onClick={handleBatchCompare}
              className="flex items-center px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded-full text-sm font-medium transition-colors"
            >
              <GitCompare size={16} className="mr-2" />
              对比选中报告
            </button>
          ) : (
            <span className="text-xs text-gray-400">{compareValidation.message}</span>
          )}
          <button
            onClick={() => {
              setSelectedReportIds(new Set());
              setAllSelected(false);
            }}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X size={18} />
          </button>
        </div>
      )}

      <UploadDialog
        isOpen={showUploadDialog}
        onClose={() => setShowUploadDialog(false)}
        onUpload={handleUpload}
      />

      {/* Table */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden">
        <table className="w-full text-left text-sm">
            <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                    <th className="px-4 py-3 font-medium text-gray-500 w-12">
                        <input
                          type="checkbox"
                          checked={allSelected}
                          onChange={handleSelectAll}
                          className="w-4 h-4 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
                        />
                    </th>
                    <th className="px-6 py-3 font-medium text-gray-500">{t('rep.id')}</th>
                    <th className="px-6 py-3 font-medium text-gray-500">{t('rep.instance')}</th>
                    <th className="px-6 py-3 font-medium text-gray-500">{t('rep.generated')}</th>
                    <th className="px-6 py-3 font-medium text-gray-500">{t('rep.period')}</th>
                    <th className="px-6 py-3 font-medium text-gray-500">{t('rep.status')}</th>
                    <th className="px-6 py-3 font-medium text-gray-500 text-right">{t('rep.actions')}</th>
                </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
                {loading ? (
                    <tr><td colSpan={7} className="p-6 text-center text-gray-500">{t('rep.loading')}</td></tr>
                ) : (
                    reports.map(report => {
                      const isSelected = selectedReportIds.has(report.id);
                      return (
                        <tr
                          key={report.id}
                          className={`hover:bg-gray-50 transition-colors ${isSelected ? 'bg-blue-50' : ''}`}
                        >
                            <td className="px-4 py-3">
                                <input
                                  type="checkbox"
                                  checked={isSelected}
                                  onChange={() => handleSelectRow(report.id)}
                                  className="w-4 h-4 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
                                />
                            </td>
                            <td className="px-6 py-3 text-gray-900 font-medium">#{report.id}</td>
                            <td className="px-6 py-3 text-gray-600">{report.instanceName}</td>
                            <td className="px-6 py-3 text-gray-600">{report.generateTime}</td>
                            <td className="px-6 py-3 text-gray-600">{report.period}</td>
                            <td className="px-6 py-3">
                                <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                                    report.status === 'Success' ? 'bg-green-100 text-green-800' :
                                    report.status === 'Failed' ? 'bg-red-100 text-red-800' : 'bg-blue-100 text-blue-800'
                                }`}>
                                    {report.status}
                                </span>
                            </td>
                            <td className="px-6 py-3 text-right space-x-2">
                                <button
                                    onClick={() => navigate(`/reports/${report.id}`)}
                                    className="text-gray-400 hover:text-blue-600 transition-colors"
                                    title="查看"
                                >
                                    <Eye size={18} />
                                </button>
                                <button
                                    onClick={() => {
                                      setSelectedReportIds(new Set([report.id]));
                                      setAllSelected(false);
                                    }}
                                    className="text-gray-400 hover:text-green-600 transition-colors"
                                    title="选择此报告进行对比"
                                >
                                    <Check size={18} />
                                </button>
                                <button
                                    onClick={() => setReportToDelete(report.id)}
                                    className="text-gray-400 hover:text-red-600 transition-colors"
                                    title="删除"
                                >
                                    <Trash2 size={18} />
                                </button>
                            </td>
                        </tr>
                      );
                    })
                )}
            </tbody>
        </table>
        {/* Pagination Mock */}
        <div className="px-6 py-3 border-t border-gray-100 flex justify-between items-center">
            <span className="text-xs text-gray-500">{t('rep.showing', {start: 1, end: 10, total: 50})}</span>
            <div className="flex space-x-1">
                <button className="px-2 py-1 border rounded hover:bg-gray-50 disabled:opacity-50">{t('rep.prev')}</button>
                <button className="px-2 py-1 border bg-blue-50 text-blue-600 border-blue-200 rounded">1</button>
                <button className="px-2 py-1 border rounded hover:bg-gray-50">2</button>
                <button className="px-2 py-1 border rounded hover:bg-gray-50">3</button>
                <button className="px-2 py-1 border rounded hover:bg-gray-50">{t('rep.next')}</button>
            </div>
        </div>
      </div>

      {/* Upload Modal - Removed, now using dialog directly */}

      {/* Delete Confirmation Modal */}
      {reportToDelete && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
              <div className="bg-white rounded-lg shadow-xl w-full max-w-sm overflow-hidden transform transition-all animate-in fade-in zoom-in-95">
                  <div className="p-6">
                      <div className="flex items-center mb-4 text-red-600">
                          <AlertTriangle size={24} className="mr-2" />
                          <h3 className="text-lg font-semibold">{t('rep.deleteTitle')}</h3>
                      </div>
                      <p className="text-gray-600 text-sm mb-6">
                          {t('rep.deleteConfirm', { id: reportToDelete })}
                      </p>
                      <div className="flex justify-end space-x-3">
                          <button
                              onClick={() => setReportToDelete(null)}
                              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                          >
                              {t('rep.cancel')}
                          </button>
                          <button
                              onClick={handleDelete}
                              className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700 transition-colors"
                          >
                              {t('rep.delete')}
                          </button>
                      </div>
                  </div>
              </div>
          </div>
      )}
    </div>
  );
};

export default ReportManagement;
