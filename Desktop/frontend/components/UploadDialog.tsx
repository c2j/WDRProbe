import React, { useState } from 'react';
import { X, FileText } from 'lucide-react';
import { useI18n } from '../context/I18nContext';
import { open } from '@tauri-apps/api/dialog';

interface UploadFormData {
  filePath: string;
  instanceName: string;
  description: string;
}

interface UploadDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onUpload: (data: UploadFormData) => Promise<void>;
}

const UploadDialog: React.FC<UploadDialogProps> = ({ isOpen, onClose, onUpload }) => {
  const { t } = useI18n();
  const [formData, setFormData] = useState<UploadFormData>({
    filePath: '',
    instanceName: '',
    description: ''
  });
  const [uploading, setUploading] = useState(false);

  const selectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'WDR Report',
            extensions: ['html', 'htm']
          }
        ]
      });

      if (selected && !Array.isArray(selected)) {
        setFormData({ ...formData, filePath: selected });
      }
    } catch (error) {
      console.error('Failed to open file dialog:', error);
      alert('Failed to open file dialog');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.filePath.trim()) {
      alert(t('rep.fileRequired'));
      return;
    }
    if (!formData.instanceName.trim()) {
      alert(t('rep.instanceRequired'));
      return;
    }

    setUploading(true);
    try {
      await onUpload(formData);
      setFormData({ filePath: '', instanceName: '', description: '' });
      onClose();
    } catch (error) {
      alert(t('rep.uploadFailed'));
    } finally {
      setUploading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-lg transform transition-all animate-in fade-in zoom-in-95">
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-800">{t('rep.importReport')}</h3>
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
              {t('rep.file')} *
            </label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                value={formData.filePath ? formData.filePath.split('/').pop() || '' : ''}
                placeholder="请选择WDR报告文件"
                readOnly
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50"
              />
              <button
                type="button"
                onClick={selectFile}
                className="px-3 py-2 bg-gray-100 hover:bg-gray-200 rounded-md text-sm font-medium text-gray-700 transition-colors flex items-center space-x-1"
              >
                <FileText size={16} />
                <span>浏览...</span>
              </button>
            </div>
            <p className="mt-1 text-xs text-gray-500">
              支持 .html, .htm 格式的 WDR 报告文件
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('rep.instanceName')} *
            </label>
            <input
              type="text"
              value={formData.instanceName}
              onChange={(e) => setFormData({ ...formData, instanceName: e.target.value })}
              placeholder="例如: prod-db-01"
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:outline-none"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('rep.description')}
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="添加报告描述..."
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:outline-none resize-none"
            />
          </div>

          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
              disabled={uploading}
            >
              {t('rep.cancel')}
            </button>
            <button
              type="submit"
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors flex items-center space-x-2 disabled:opacity-50"
              disabled={uploading || !formData.filePath}
            >
              {uploading && (
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              )}
              <span>{uploading ? t('rep.uploading') : t('rep.import')}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default UploadDialog;
