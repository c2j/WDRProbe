
import React, { useEffect, useState, useMemo } from 'react';
import { BookOpen, Search, AlertCircle, Loader2, Filter } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { ApiService } from '../services/apiService';
import type { RuleInfo } from '../types';

const categoryColors: Record<string, string> = {
  Cost: 'bg-blue-50 text-blue-700 border-blue-200',
  Join: 'bg-indigo-50 text-indigo-700 border-indigo-200',
  Scan: 'bg-teal-50 text-teal-700 border-teal-200',
  Parallel: 'bg-cyan-50 text-cyan-700 border-cyan-200',
  Memory: 'bg-violet-50 text-violet-700 border-violet-200',
  Misc: 'bg-gray-50 text-gray-700 border-gray-200',
};

const severityColors: Record<string, string> = {
  Critical: 'bg-red-100 text-red-700 border-red-200',
  Warning: 'bg-yellow-100 text-yellow-700 border-yellow-200',
  Info: 'bg-blue-100 text-blue-700 border-blue-200',
};

const DiagnosticRules: React.FC = () => {
  const { t } = useI18n();
  const [rules, setRules] = useState<RuleInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const result = await ApiService.listDiagnosticRules();
        setRules(result);
      } catch (e: any) {
        setError(e.message || t('vis.diagnostic.error', { msg: 'Unknown error' }));
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const categories = useMemo(() => {
    const set = new Set(rules.map(r => r.category));
    return Array.from(set).sort();
  }, [rules]);

  const filtered = useMemo(() => {
    return rules.filter(r => {
      const matchSearch = !search || 
        r.ruleId.toLowerCase().includes(search.toLowerCase()) ||
        r.title.toLowerCase().includes(search.toLowerCase()) ||
        r.category.toLowerCase().includes(search.toLowerCase());
      const matchCategory = !selectedCategory || r.category === selectedCategory;
      return matchSearch && matchCategory;
    });
  }, [rules, search, selectedCategory]);

  return (
    <div className="space-y-6">
      <div className="flex items-center space-x-3">
        <div className="p-2 bg-gradient-to-br from-emerald-500 to-teal-500 rounded-lg shadow-md">
          <BookOpen className="text-white" size={24} />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-gray-800">{t('vis.diagnostic.rulesCatalog')}</h1>
          <p className="text-sm text-gray-500 mt-1">
            {loading
              ? t('vis.diagnostic.loading')
              : t('vis.diagnostic.rulesCount', { count: rules.length })}
          </p>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4 flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t('vis.diagnostic.searchRules')}
            className="w-full pl-9 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
          />
        </div>
        <div className="flex items-center space-x-2 overflow-x-auto">
          <Filter size={16} className="text-gray-400 shrink-0" />
          <button
            onClick={() => setSelectedCategory('')}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors whitespace-nowrap ${
              selectedCategory === ''
                ? 'bg-blue-50 text-blue-700 border-blue-200'
                : 'bg-white text-gray-600 border-gray-200 hover:bg-gray-50'
            }`}
          >
            {t('vis.diagnostic.allCategories')}
          </button>
          {categories.map(cat => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors whitespace-nowrap ${
                selectedCategory === cat
                  ? 'bg-blue-50 text-blue-700 border-blue-200'
                  : 'bg-white text-gray-600 border-gray-200 hover:bg-gray-50'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-xl p-4 flex items-start space-x-3">
          <AlertCircle className="text-red-500 shrink-0 mt-0.5" size={18} />
          <span className="text-red-700 text-sm">{error}</span>
        </div>
      )}

      {loading && (
        <div className="flex items-center justify-center py-20">
          <Loader2 size={32} className="animate-spin text-blue-500" />
          <span className="ml-3 text-gray-500">{t('vis.diagnostic.loading')}</span>
        </div>
      )}

      {!loading && !error && filtered.length === 0 && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-12 text-center">
          <BookOpen size={40} className="mx-auto text-gray-300 mb-3" />
          <p className="text-gray-500">{t('vis.diagnostic.noRules')}</p>
        </div>
      )}

      {!loading && filtered.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {filtered.map((rule) => (
            <div
              key={rule.ruleId}
              className="bg-white rounded-xl shadow-sm border border-gray-200 p-5 hover:shadow-md hover:border-gray-300 transition-all"
            >
              <div className="flex items-start justify-between mb-3">
                <span className="font-mono text-[11px] font-semibold text-gray-400 bg-gray-100 px-2 py-0.5 rounded">
                  {rule.ruleId}
                </span>
                <span
                  className={`inline-block px-2.5 py-0.5 rounded-full text-xs font-medium border ${
                    severityColors[rule.severity] || 'bg-gray-100 text-gray-700 border-gray-200'
                  }`}
                >
                  {rule.severity}
                </span>
              </div>
              <span
                className={`inline-block px-2 py-0.5 rounded text-[11px] font-medium border mb-2 ${
                  categoryColors[rule.category] || 'bg-gray-50 text-gray-600 border-gray-200'
                }`}
              >
                {rule.category}
              </span>
              <h3 className="font-semibold text-gray-800 text-sm mt-1 leading-snug">{rule.title}</h3>
              <p className="text-xs text-gray-500 mt-2 leading-relaxed">{rule.description}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default DiagnosticRules;
