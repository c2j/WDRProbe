import React, { useState } from 'react';
import { X, FileCode, AlertCircle } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { open } from '@tauri-apps/api/dialog';
import { readTextFile } from '@tauri-apps/api/fs';

interface PlanImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onImport: (planText: string, format: 'json' | 'text') => Promise<void>;
}

const PlanImportDialog: React.FC<PlanImportDialogProps> = ({ isOpen, onClose, onImport }) => {
  const { t } = useI18n();
  const [planText, setPlanText] = useState('');
  const [format, setFormat] = useState<'json' | 'text'>('json');
  const [filePath, setFilePath] = useState('');
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Execution Plan',
            extensions: ['json', 'txt', 'log']
          },
          {
            name: 'All Files',
            extensions: ['*']
          }
        ]
      });

      if (selected && !Array.isArray(selected)) {
        setFilePath(selected);
        setError(null);

        // Read file content
        try {
          const content = await readTextFile(selected);
          setPlanText(content);

          // Auto-detect format
          if (selected.endsWith('.json') || content.trim().startsWith('{')) {
            setFormat('json');
          } else {
            setFormat('text');
          }
        } catch (readError) {
          setError(t('vis.import.readError'));
        }
      }
    } catch (error) {
      console.error('Failed to open file dialog:', error);
      setError(t('vis.import.openFailed'));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!planText.trim()) {
      setError(t('vis.import.planRequired'));
      return;
    }

    setImporting(true);
    setError(null);
    try {
      await onImport(planText, format);
      setPlanText('');
      setFilePath('');
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('vis.import.importFailed'));
    } finally {
      setImporting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl transform transition-all animate-in fade-in zoom-in-95">
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-800">{t('vis.import.title')}</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('vis.import.file')} *
            </label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                value={filePath ? filePath.split('/').pop() || '' : ''}
                placeholder={t('vis.import.noFile')}
                readOnly
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50"
              />
              <button
                type="button"
                onClick={selectFile}
                className="px-3 py-2 bg-gray-100 hover:bg-gray-200 rounded-md text-sm font-medium text-gray-700 transition-colors flex items-center space-x-1"
              >
                <FileCode size={16} />
                <span>{t('vis.import.browse')}</span>
              </button>
            </div>
            <p className="mt-1 text-xs text-gray-500">
              {t('vis.import.fileHint')}
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('vis.import.format')} *
            </label>
            <div className="flex items-center space-x-4">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="radio"
                  name="format"
                  value="json"
                  checked={format === 'json'}
                  onChange={(e) => setFormat(e.target.value as 'json' | 'text')}
                  className="text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-700">JSON</span>
              </label>
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="radio"
                  name="format"
                  value="text"
                  checked={format === 'text'}
                  onChange={(e) => setFormat(e.target.value as 'json' | 'text')}
                  className="text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-700">{t('vis.import.textFormat')}</span>
              </label>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('vis.import.planText')}
            </label>
            <textarea
              value={planText}
              onChange={(e) => setPlanText(e.target.value)}
              placeholder={t('vis.import.textPlaceholder')}
              rows={10}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:outline-none resize-none"
            />
          </div>

          {error && (
            <div className="flex items-start p-3 bg-red-50 rounded-md border border-red-100">
              <AlertCircle size={16} className="text-red-500 mr-2 mt-0.5 flex-shrink-0" />
              <p className="text-sm text-red-700">{error}</p>
            </div>
          )}

          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
              disabled={importing}
            >
              {t('rep.cancel')}
            </button>
            <button
              type="submit"
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors flex items-center space-x-2 disabled:opacity-50"
              disabled={importing || !planText.trim()}
            >
              {importing && (
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              )}
              <span>{importing ? t('vis.import.importing') : t('vis.import.importBtn')}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default PlanImportDialog;
