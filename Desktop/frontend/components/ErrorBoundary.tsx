import { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary Component
 * Catches JavaScript errors anywhere in the child component tree,
 * logs those errors, and displays a fallback UI.
 *
 * Per Constitution: User Story 10 - Cross-Cutting Concerns
 */
class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    };
  }

  static getDerivedStateFromError(_error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Log the error to console for debugging
    console.error('Error Boundary caught an error:', error);
    console.error('Error Info:', errorInfo);

    // Store error info in state for display
    this.setState({
      error,
      errorInfo
    });

    // TODO: Send error to logging service in production
    // this.logErrorToService(error, errorInfo);
  }

  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    });
  };

  handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
          <div className="max-w-2xl w-full bg-white rounded-lg shadow-lg border border-gray-200 p-8">
            {/* Error Icon */}
            <div className="flex items-center justify-center mb-6">
              <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center">
                <AlertTriangle size={32} className="text-red-600" />
              </div>
            </div>

            {/* Error Title */}
            <h1 className="text-2xl font-bold text-gray-900 text-center mb-2">
              Something went wrong
            </h1>
            <p className="text-gray-600 text-center mb-6">
              An unexpected error occurred. Please try refreshing the page or contact support if the problem persists.
            </p>

            {/* Error Details (Development) */}
            {import.meta.env.DEV && this.state.error && (
              <div className="mb-6">
                <details className="bg-gray-50 rounded border border-gray-200">
                  <summary className="px-4 py-3 cursor-pointer text-sm font-medium text-gray-700 hover:bg-gray-100">
                    Error Details (Development Only)
                  </summary>
                  <div className="px-4 pb-4">
                    <div className="mt-3">
                      <h4 className="text-xs font-semibold text-gray-500 uppercase mb-1">Error Message</h4>
                      <pre className="text-xs bg-red-50 text-red-700 p-2 rounded overflow-auto max-h-32">
                        {this.state.error.toString()}
                      </pre>
                    </div>
                    {this.state.errorInfo && (
                      <div className="mt-3">
                        <h4 className="text-xs font-semibold text-gray-500 uppercase mb-1">Component Stack</h4>
                        <pre className="text-xs bg-gray-100 text-gray-700 p-2 rounded overflow-auto max-h-32">
                          {this.state.errorInfo.componentStack}
                        </pre>
                      </div>
                    )}
                  </div>
                </details>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <button
                onClick={this.handleReset}
                className="flex items-center justify-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium"
              >
                <RefreshCw size={16} className="mr-2" />
                Try Again
              </button>
              <button
                onClick={this.handleReload}
                className="flex items-center justify-center px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors font-medium"
              >
                Reload Page
              </button>
            </div>

            {/* Support Info */}
            <div className="mt-6 pt-6 border-t border-gray-200 text-center text-sm text-gray-500">
              <p>If this error persists, please contact our support team with the error details above.</p>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
