import React, { ReactNode, useState, useRef, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import {
  LayoutDashboard,
  FileText,
  GitCompare,
  Settings,
  ShieldAlert,
  History,
  Bell,
  User,
  ChevronLeft,
  ChevronRight,
  GitBranch,
  Languages,
  Download,
  Upload,
  Menu
} from 'lucide-react';
import { useI18n } from '../context/I18nContext';

interface LayoutProps {
  children: ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [importExportMenuOpen, setImportExportMenuOpen] = useState(false);
  const location = useLocation();
  const { t, language, setLanguage } = useI18n();
  const importExportRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (importExportRef.current && !importExportRef.current.contains(event.target as Node)) {
        setImportExportMenuOpen(false);
      }
    };

    if (importExportMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [importExportMenuOpen]);

  // Import/Export PNG handlers
  const handleImportPNG = async () => {
    try {
      // Open file dialog for PNG import
      const { open } = await import('@tauri-apps/api/dialog');
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'PNG Image',
          extensions: ['png']
        }]
      });
      if (selected && typeof selected === 'string') {
        console.log('Import PNG:', selected);
        // Handle PNG import logic here
      }
    } catch (error) {
      console.error('Failed to import PNG:', error);
    }
    setImportExportMenuOpen(false);
  };

  const handleExportPNG = async () => {
    try {
      // Get the main content area to capture
      const mainContent = document.querySelector('main');
      if (!mainContent) {
        console.warn('No main content found to export');
        setImportExportMenuOpen(false);
        return;
      }

      // Try to use html2canvas for PNG export
      const html2canvas = await import('html2canvas');
      const canvas = await html2canvas.default(mainContent as HTMLElement, {
        backgroundColor: '#f9fafb',
        scale: 2, // Higher quality
      });

      // Convert canvas to blob
      canvas.toBlob(async (blob) => {
        if (blob) {
          // Use Tauri dialog to save the file
          const { save } = await import('@tauri-apps/api/dialog');

          const filePath = await save({
            filters: [{
              name: 'PNG Image',
              extensions: ['png']
            }],
            defaultPath: `wdrprobe-export-${new Date().toISOString().slice(0, 10)}.png`
          });

          if (filePath) {
            // Convert blob to base64 and use download as fallback
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filePath.split('/').pop() || 'export.png';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            console.log('PNG exported successfully:', filePath);
          }
        }
      }, 'image/png');
    } catch (error) {
      console.error('Failed to export PNG:', error);
      alert(language === 'zh'
        ? '导出失败：请确保已安装 html2canvas 库'
        : 'Export failed: Please ensure html2canvas library is installed');
    }
    setImportExportMenuOpen(false);
  };

  const MENU_ITEMS = [
    { path: '/', labelKey: 'menu.dashboard', icon: LayoutDashboard },
    { path: '/reports', labelKey: 'menu.reports', icon: FileText },
    { path: '/comparison', labelKey: 'menu.comparison', icon: GitCompare },
    { path: '/visualizer', labelKey: 'menu.visualizer', icon: GitBranch },
    { path: '/thresholds', labelKey: 'menu.thresholds', icon: Settings },
    { path: '/sqlaudit', labelKey: 'menu.sqlaudit', icon: ShieldAlert },
    { path: '/auditlog', labelKey: 'menu.auditlog', icon: History },
  ];

  const currentLabel = MENU_ITEMS.find(m => m.path === location.pathname)?.labelKey;

  return (
    <div className="flex h-screen bg-gray-100 overflow-hidden">
      {/* Sidebar */}
      <aside 
        className={`${
          sidebarOpen ? 'w-64' : 'w-20'
        } bg-[#0f2c4b] text-white transition-all duration-300 ease-in-out flex flex-col shadow-xl z-20`}
      >
        {/* Logo Area */}
        <div className="h-16 flex items-center justify-center border-b border-gray-700">
          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center font-bold text-lg">W</div>
            {sidebarOpen && <span className="font-bold text-lg tracking-wide truncate">WDRProbe</span>}
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-4">
          <ul className="space-y-1 px-2">
            {MENU_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.path;
              return (
                <li key={item.path}>
                  <Link
                    to={item.path}
                    className={`flex items-center px-4 py-3 rounded-md transition-colors ${
                      isActive 
                        ? 'bg-blue-600 text-white shadow-md' 
                        : 'text-gray-300 hover:bg-gray-800 hover:text-white'
                    }`}
                  >
                    <Icon size={20} className="min-w-[20px]" />
                    {sidebarOpen && <span className="ml-3 truncate">{t(item.labelKey)}</span>}
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>

        {/* Toggle Button */}
        <div className="p-4 border-t border-gray-700">
            <button 
                onClick={() => setSidebarOpen(!sidebarOpen)}
                className="w-full flex items-center justify-center p-2 rounded-md hover:bg-gray-800 transition-colors"
            >
                {sidebarOpen ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
            </button>
        </div>
      </aside>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Top Navbar */}
        <header className="h-16 bg-white shadow-sm flex items-center justify-between px-6 z-10">
          <div className="text-gray-500 text-sm">
             <span className="font-semibold text-gray-700">WDRProbe</span> / {currentLabel ? t(currentLabel) : 'Page'}
          </div>

          <div className="flex items-center space-x-6">
            {/* Import/Export PNG Menu - Per Constitution VIII */}
            <div className="relative" ref={importExportRef}>
              <button
                onClick={() => setImportExportMenuOpen(!importExportMenuOpen)}
                className="text-gray-500 hover:text-blue-600 flex items-center space-x-1 px-2 py-1 rounded hover:bg-gray-50 transition-colors"
                title={language === 'zh' ? '导入/导出PNG' : 'Import/Export PNG'}
              >
                <Menu size={20} />
                <span className="text-sm font-medium">{language === 'zh' ? 'PNG' : 'PNG'}</span>
              </button>

              {importExportMenuOpen && (
                <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg border border-gray-200 py-1 z-50">
                  <button
                    onClick={handleImportPNG}
                    className="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2"
                  >
                    <Upload size={16} />
                    <span>{language === 'zh' ? '导入PNG' : 'Import PNG'}</span>
                  </button>
                  <button
                    onClick={handleExportPNG}
                    className="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2"
                  >
                    <Download size={16} />
                    <span>{language === 'zh' ? '导出PNG' : 'Export PNG'}</span>
                  </button>
                </div>
              )}
            </div>

            {/* Language Switcher */}
            <button
                onClick={() => setLanguage(language === 'zh' ? 'en' : 'zh')}
                className="text-gray-500 hover:text-blue-600 flex items-center space-x-1 px-2 py-1 rounded hover:bg-gray-50 transition-colors"
                title="Switch Language"
            >
                <Languages size={20} />
                <span className="text-sm font-medium">{language === 'zh' ? 'EN' : '中文'}</span>
            </button>

            <div className="relative cursor-pointer text-gray-500 hover:text-blue-600">
                <Bell size={20} />
                <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">3</span>
            </div>
            <div className="flex items-center space-x-2 cursor-pointer hover:bg-gray-50 p-2 rounded-md">
                <div className="w-8 h-8 bg-blue-100 text-blue-600 rounded-full flex items-center justify-center">
                    <User size={16} />
                </div>
                <div className="text-sm">
                    <p className="font-medium text-gray-700">Zhang San</p>
                    <p className="text-xs text-gray-400">{t('header.role')}</p>
                </div>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main className="flex-1 overflow-auto p-6 bg-gray-50">
          {children}
        </main>
      </div>
    </div>
  );
};

export default Layout;
