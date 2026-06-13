import React, { useEffect, useState } from 'react';
import { ApiService, rewriteSql } from '../services/apiService';
import { SqlAuditIssue, RewriteOutput } from '../types';
import { X, AlertTriangle, CheckCircle, Code, Zap, FileText } from 'lucide-react';
import { useI18n } from '../context/I18nContext';

const SqlAuditPage: React.FC = () => {
  const { t } = useI18n();
  const [issues, setIssues] = useState<SqlAuditIssue[]>([]);
  const [selectedIssue, setSelectedIssue] = useState<SqlAuditIssue | null>(null);
  const [rewriteInput, setRewriteInput] = useState('');
  const [schemaInput, setSchemaInput] = useState('');
  const [rewriteResult, setRewriteResult] = useState<RewriteOutput | null>(null);
  const [rewriteLoading, setRewriteLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    ApiService.getSqlAuditIssues().then(setIssues);
  }, []);

  const closeLoop = () => setSelectedIssue(null);

  const handleRewrite = async () => {
    setRewriteLoading(true);
    try {
      const result = await rewriteSql(rewriteInput, undefined, schemaInput || undefined);
      setRewriteResult(result);
    } catch (err) {
      console.error('Rewrite error:', err);
    } finally {
      setRewriteLoading(false);
    }
  };

  const STATUS_KEYS: Record<string, string> = {
    'All': 'audit.all',
    'Pending': 'audit.pending',
    'Processing': 'audit.processing',
    'Fixed': 'audit.fixed',
    'Whitelisted': 'audit.whitelisted'
  };

  return (
    <div className="space-y-4">
        <div className="flex space-x-2 pb-4 border-b border-gray-200">
             {Object.keys(STATUS_KEYS).map(status => (
                 <button key={status} className={`px-4 py-1.5 rounded-full text-sm ${status === 'All' ? 'bg-gray-800 text-white' : 'bg-white border border-gray-300 text-gray-600 hover:bg-gray-50'}`}>
                     {t(STATUS_KEYS[status])}
                 </button>
             ))}
        </div>

        <div className="bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden">
            <table className="w-full text-left text-sm">
                <thead className="bg-gray-50 border-b border-gray-200">
                    <tr>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.id')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.severity')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.type')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.target')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.foundTime')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500">{t('audit.status')}</th>
                        <th className="px-6 py-3 font-medium text-gray-500 text-right">{t('audit.actions')}</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                    {issues.map(issue => (
                        <tr key={issue.id} className="hover:bg-gray-50">
                             <td className="px-6 py-3 font-mono text-gray-600">{issue.id}</td>
                             <td className="px-6 py-3">
                                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${
                                    issue.severity === 'High' ? 'bg-red-100 text-red-700' : 'bg-yellow-100 text-yellow-700'
                                }`}>
                                    {issue.severity}
                                </span>
                             </td>
                             <td className="px-6 py-3 text-gray-700">{issue.type}</td>
                             <td className="px-6 py-3 text-gray-500 truncate max-w-xs" title={issue.target}>{issue.target}</td>
                             <td className="px-6 py-3 text-gray-500">{issue.time}</td>
                             <td className="px-6 py-3 text-gray-700">{issue.status}</td>
                             <td className="px-6 py-3 text-right">
                                 <button 
                                    onClick={() => setSelectedIssue(issue)}
                                    className="text-blue-600 hover:underline font-medium"
                                 >
                                    {t('audit.optimize')}
                                 </button>
                             </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>

        {/* Optimization Modal */}
        {selectedIssue && (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
                <div className="bg-white rounded-xl shadow-2xl w-full max-w-2xl overflow-hidden transform transition-all animate-in fade-in zoom-in-95 duration-200 flex flex-col max-h-[90vh]">
                    
                    {/* Header */}
                    <div className="px-6 py-4 border-b border-gray-100 flex justify-between items-center bg-gray-50">
                        <div className="flex items-center space-x-3">
                            <div className="p-2 bg-blue-100 text-blue-600 rounded-lg">
                                <Zap size={20} />
                            </div>
                            <div>
                                <h3 className="font-semibold text-gray-800 text-lg">{t('audit.modalTitle')}</h3>
                                <p className="text-xs text-gray-500">{t('audit.issueId')}: {selectedIssue.id}</p>
                            </div>
                        </div>
                        <button 
                            onClick={closeLoop} 
                            className="text-gray-400 hover:text-gray-600 transition-colors rounded-full p-1 hover:bg-gray-200"
                        >
                            <X size={20} />
                        </button>
                    </div>

                    {/* Body */}
                    <div className="p-6 overflow-y-auto space-y-6">
                        {/* Issue Summary */}
                        <div className="flex items-center space-x-4">
                             <div className={`px-3 py-1 rounded-full text-sm font-medium border ${
                                selectedIssue.severity === 'High' 
                                    ? 'bg-red-50 text-red-700 border-red-100' 
                                    : 'bg-yellow-50 text-yellow-700 border-yellow-100'
                             }`}>
                                {selectedIssue.severity} Severity
                             </div>
                             <div className="text-sm text-gray-500">
                                Type: <span className="font-medium text-gray-700">{selectedIssue.type}</span>
                             </div>
                             <div className="text-sm text-gray-500">
                                Detected: <span className="font-medium text-gray-700">{selectedIssue.time}</span>
                             </div>
                        </div>

                        {/* SQL Statement */}
                        <div>
                            <h4 className="text-sm font-semibold text-gray-700 mb-2 flex items-center">
                                <Code size={16} className="mr-2 text-gray-400" /> {t('audit.targetSql')}
                            </h4>
                            <div className="bg-gray-800 rounded-lg p-4 font-mono text-sm text-gray-200 overflow-x-auto shadow-inner border border-gray-700">
                                {selectedIssue.target}
                            </div>
                        </div>

                        {/* Diagnosis & Suggestion (Mocked for demo) */}
                        <div className="space-y-4">
                            <div className="bg-blue-50 border border-blue-100 rounded-lg p-4">
                                <h4 className="text-sm font-semibold text-blue-800 mb-1 flex items-center">
                                    <FileText size={16} className="mr-2" /> {t('audit.diagnosis')}
                                </h4>
                                <p className="text-sm text-blue-700 leading-relaxed">
                                    This query is performing a full table scan on table <code className="font-mono font-bold">t_order</code> which contains 1.2M rows. The filtering condition on column <code className="font-mono font-bold">create_time</code> is not utilizing any existing indexes.
                                </p>
                            </div>
                            
                            <div className="bg-green-50 border border-green-100 rounded-lg p-4">
                                <h4 className="text-sm font-semibold text-green-800 mb-1 flex items-center">
                                    <CheckCircle size={16} className="mr-2" /> {t('audit.recommendation')}
                                </h4>
                                <p className="text-sm text-green-700 leading-relaxed mb-3">
                                    Create a composite index on <code className="font-mono font-bold">(create_time, status)</code> to optimize range queries and filtering.
                                </p>
                                <div className="bg-white border border-green-200 rounded p-3 font-mono text-xs text-gray-600">
                                    CREATE INDEX idx_order_create_time ON t_order(create_time, status);
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Footer */}
                    <div className="px-6 py-4 bg-gray-50 border-t border-gray-100 flex justify-end space-x-3">
                        <button 
                            onClick={closeLoop}
                            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                        >
                            {t('audit.cancel')}
                        </button>
                        <button 
                            onClick={() => { alert('Added to Whitelist'); closeLoop(); }}
                            className="px-4 py-2 text-sm font-medium text-red-600 bg-red-50 border border-transparent rounded-md hover:bg-red-100 transition-colors"
                        >
                            {t('audit.whitelist')}
                        </button>
                        <button 
                            onClick={() => { alert('Optimization applied (simulated)'); closeLoop(); }}
                            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 shadow-sm transition-colors flex items-center"
                        >
                            <Zap size={16} className="mr-2" />
                            {t('audit.apply')}
                        </button>
                    </div>
                </div>
            </div>
        )}

        {/* SQL Rewrite Panel */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-100 p-6 mt-6">
          <h2 className="text-xl font-bold mb-4">{t('rewrite.title')}</h2>
          <p className="text-sm text-gray-500 mb-4">{t('rewrite.description')}</p>
          
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium mb-1">{t('rewrite.inputLabel')}</label>
              <textarea
                className="w-full h-32 p-3 border rounded font-mono text-sm"
                placeholder={t('rewrite.inputPlaceholder')}
                value={rewriteInput}
                onChange={(e) => setRewriteInput(e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">{t('rewrite.schemaLabel')}</label>
              <textarea
                className="w-full h-20 p-3 border rounded font-mono text-xs"
                placeholder={t('rewrite.schemaPlaceholder')}
                value={schemaInput}
                onChange={(e) => setSchemaInput(e.target.value)}
              />
            </div>
            <button
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              onClick={handleRewrite}
              disabled={!rewriteInput.trim() || rewriteLoading}
            >
              {rewriteLoading ? t('rewrite.loading') : t('rewrite.button')}
            </button>
          </div>

          {rewriteResult && (
            <div className="mt-6 space-y-4">
              {rewriteResult.changed ? (
                <>
                  <div className="p-3 bg-green-50 text-green-700 rounded border border-green-200">
                    {t('rewrite.changed')}
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <h3 className="font-semibold mb-2">{t('rewrite.original')}</h3>
                      <pre className="p-3 bg-gray-100 rounded text-xs font-mono overflow-auto">
                        {rewriteResult.original_sql}
                      </pre>
                    </div>
                    <div>
                      <h3 className="font-semibold mb-2 text-green-600">{t('rewrite.rewritten')}</h3>
                      <pre className="p-3 bg-green-50 rounded text-xs font-mono overflow-auto border border-green-200">
                        {rewriteResult.rewritten_sql}
                      </pre>
                    </div>
                  </div>
                  
                  <button
                    className="px-3 py-1 text-sm border rounded hover:bg-gray-100"
                    onClick={() => {
                      navigator.clipboard.writeText(rewriteResult.rewritten_sql);
                      setCopied(true);
                      setTimeout(() => setCopied(false), 2000);
                    }}
                  >
                    {copied ? t('rewrite.copied') : t('rewrite.copyRewritten')}
                  </button>
                </>
              ) : (
                <div className="p-3 bg-gray-100 text-gray-500 rounded">
                  {t('rewrite.noChange')}
                </div>
              )}

              {rewriteResult.suggestions.length > 0 && (
                <div>
                  <h3 className="font-semibold mb-2">{t('rewrite.suggestions')}</h3>
                  {rewriteResult.suggestions.map((s, i) => (
                    <div key={i} className="border rounded p-3 mb-2">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-mono text-xs bg-gray-200 px-2 py-0.5 rounded">{s.rule_id}</span>
                        <span className={`text-xs px-2 py-0.5 rounded ${
                          s.confidence === 'High' ? 'bg-green-200 text-green-800' :
                          s.confidence === 'Medium' ? 'bg-yellow-200 text-yellow-800' :
                          'bg-gray-200 text-gray-800'
                        }`}>
                          {s.confidence}
                        </span>
                      </div>
                      <p className="text-sm">{s.rule_description}</p>
                      {s.notes.map((note, j) => (
                        <p key={j} className="text-xs text-gray-500 mt-1">• {note}</p>
                      ))}
                    </div>
                  ))}
                </div>
              )}

              {rewriteResult.match_failures.length > 0 && (
                <details className="border rounded p-3">
                  <summary className="cursor-pointer text-sm font-medium">{t('rewrite.matchFailures')}</summary>
                  <div className="mt-2 space-y-1">
                    {rewriteResult.match_failures.map((f, i) => (
                      <div key={i} className="text-xs flex gap-2">
                        <span className="font-mono text-gray-600">{f.rule_id}:</span>
                        <span className="text-gray-500">{f.reason}</span>
                      </div>
                    ))}
                  </div>
                </details>
              )}
            </div>
          )}
        </div>
    </div>
  );
};

export default SqlAuditPage;
