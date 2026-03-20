import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MarketplacePlugin {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  download_url: string;
  api_version: number;
  plugin_type: string;
  downloads: number;
  rating: number;
}

interface InstalledPlugin {
  id: string;
  name: string;
  version: string;
  type: string;
  status: string;
}

type TabType = 'installed' | 'marketplace';

const PluginManager: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabType>('marketplace');
  const [marketplacePlugins, setMarketplacePlugins] = useState<MarketplacePlugin[]>([]);
  const [installedPlugins, setInstalledPlugins] = useState<InstalledPlugin[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installingPlugins, setInstallingPlugins] = useState<Set<string>>(new Set());

  // Load marketplace plugins
  const loadMarketplacePlugins = async () => {
    try {
      setLoading(true);
      setError(null);
      const plugins = await invoke<MarketplacePlugin[]>('get_marketplace_plugins');
      setMarketplacePlugins(plugins);
    } catch (err) {
      setError(`Failed to load marketplace: ${err}`);
      console.error('Failed to load marketplace:', err);
    } finally {
      setLoading(false);
    }
  };

  // Load installed plugins
  const loadInstalledPlugins = async () => {
    try {
      setLoading(true);
      setError(null);
      const plugins = await invoke<InstalledPlugin[]>('get_plugins');
      setInstalledPlugins(plugins);
    } catch (err) {
      setError(`Failed to load installed plugins: ${err}`);
      console.error('Failed to load installed plugins:', err);
    } finally {
      setLoading(false);
    }
  };

  // Search marketplace
  const searchMarketplace = async (query: string) => {
    if (!query.trim()) {
      loadMarketplacePlugins();
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const results = await invoke<MarketplacePlugin[]>('search_marketplace', { query });
      setMarketplacePlugins(results);
    } catch (err) {
      setError(`Search failed: ${err}`);
      console.error('Search failed:', err);
    } finally {
      setLoading(false);
    }
  };

  // Install plugin
  const installPlugin = async (pluginId: string) => {
    try {
      setInstallingPlugins((prev) => new Set(prev).add(pluginId));
      await invoke('install_plugin', { pluginId });

      // Reload both lists
      await loadInstalledPlugins();
      await loadMarketplacePlugins();

      setError(null);
    } catch (err) {
      setError(`Failed to install plugin: ${err}`);
      console.error('Failed to install plugin:', err);
    } finally {
      setInstallingPlugins((prev) => {
        const next = new Set(prev);
        next.delete(pluginId);
        return next;
      });
    }
  };

  // Uninstall plugin
  const uninstallPlugin = async (pluginId: string) => {
    if (!confirm(`Are you sure you want to uninstall this plugin?`)) {
      return;
    }

    try {
      await invoke('uninstall_plugin', { pluginId });
      await loadInstalledPlugins();
      await loadMarketplacePlugins();
      setError(null);
    } catch (err) {
      setError(`Failed to uninstall plugin: ${err}`);
      console.error('Failed to uninstall plugin:', err);
    }
  };

  // Load plugins on mount
  useEffect(() => {
    loadMarketplacePlugins();
    loadInstalledPlugins();
  }, []);

  // Handle search
  useEffect(() => {
    const timer = setTimeout(() => {
      if (activeTab === 'marketplace') {
        searchMarketplace(searchQuery);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [searchQuery, activeTab]);

  // Check if plugin is installed
  const isPluginInstalled = (pluginId: string): boolean => {
    return installedPlugins.some((p) => p.id === pluginId);
  };

  // Render star rating
  const renderStars = (rating: number) => {
    const stars = [];
    for (let i = 0; i < 5; i++) {
      if (i < Math.floor(rating)) {
        stars.push(<span key={i}>★</span>);
      } else if (i < rating) {
        stars.push(<span key={i}>☆</span>);
      } else {
        stars.push(<span key={i}>☆</span>);
      }
    }
    return <span className="text-yellow-400">{stars}</span>;
  };

  // Render plugin type badge
  const renderTypeBadge = (type: string) => {
    const colors: Record<string, string> = {
      image_filter: 'bg-blue-500',
      integration: 'bg-green-500',
      ai_ml: 'bg-purple-500',
      import_export: 'bg-orange-500',
      metadata: 'bg-cyan-500',
      ui_panel: 'bg-pink-500',
      catalog: 'bg-indigo-500',
      tethering: 'bg-red-500',
    };

    const color = colors[type] || 'bg-gray-500';

    return (
      <span className={`${color} text-white text-xs px-2 py-1 rounded`}>
        {type.replace('_', ' ')}
      </span>
    );
  };

  return (
    <div className="h-full flex flex-col bg-gray-900 text-white">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <h1 className="text-2xl font-bold mb-4">Plugin Manager</h1>

        {/* Tabs */}
        <div className="flex space-x-4 mb-4">
          <button
            className={`px-4 py-2 rounded ${
              activeTab === 'installed'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
            onClick={() => setActiveTab('installed')}
          >
            Installed ({installedPlugins.length})
          </button>
          <button
            className={`px-4 py-2 rounded ${
              activeTab === 'marketplace'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
            onClick={() => setActiveTab('marketplace')}
          >
            Marketplace ({marketplacePlugins.length})
          </button>
        </div>

        {/* Search */}
        <input
          type="text"
          placeholder="Search plugins..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded focus:outline-none focus:border-blue-500"
        />
      </div>

      {/* Error message */}
      {error && (
        <div className="p-4 bg-red-900 border-b border-red-700">
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading && (
          <div className="text-center py-8">
            <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
            <p className="mt-2 text-gray-400">Loading...</p>
          </div>
        )}

        {!loading && activeTab === 'installed' && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {installedPlugins.length === 0 ? (
              <div className="col-span-full text-center py-8 text-gray-400">
                No plugins installed. Visit the Marketplace to install plugins.
              </div>
            ) : (
              installedPlugins.map((plugin) => (
                <div
                  key={plugin.id}
                  className="bg-gray-800 rounded-lg p-4 border border-gray-700 hover:border-gray-600"
                >
                  <div className="flex justify-between items-start mb-2">
                    <h3 className="text-lg font-semibold">{plugin.name}</h3>
                    {renderTypeBadge(plugin.type)}
                  </div>
                  <p className="text-sm text-gray-400 mb-2">v{plugin.version}</p>
                  <div className="flex justify-between items-center mt-4">
                    <span
                      className={`text-xs px-2 py-1 rounded ${
                        plugin.status === 'active'
                          ? 'bg-green-900 text-green-200'
                          : 'bg-gray-700 text-gray-300'
                      }`}
                    >
                      {plugin.status}
                    </span>
                    <button
                      onClick={() => uninstallPlugin(plugin.id)}
                      className="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm"
                    >
                      Uninstall
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {!loading && activeTab === 'marketplace' && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {marketplacePlugins.length === 0 ? (
              <div className="col-span-full text-center py-8 text-gray-400">
                No plugins found.
              </div>
            ) : (
              marketplacePlugins.map((plugin) => {
                const installed = isPluginInstalled(plugin.id);
                const installing = installingPlugins.has(plugin.id);

                return (
                  <div
                    key={plugin.id}
                    className="bg-gray-800 rounded-lg p-4 border border-gray-700 hover:border-gray-600"
                  >
                    <div className="flex justify-between items-start mb-2">
                      <h3 className="text-lg font-semibold">{plugin.name}</h3>
                      {renderTypeBadge(plugin.plugin_type)}
                    </div>
                    <p className="text-sm text-gray-400 mb-2">{plugin.description}</p>
                    <p className="text-xs text-gray-500 mb-2">by {plugin.author}</p>

                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center space-x-2">
                        {renderStars(plugin.rating)}
                        <span className="text-xs text-gray-400">({plugin.rating})</span>
                      </div>
                      <span className="text-xs text-gray-400">
                        {plugin.downloads.toLocaleString()} downloads
                      </span>
                    </div>

                    <div className="flex justify-between items-center mt-4">
                      <span className="text-xs text-gray-500">v{plugin.version}</span>
                      {installed ? (
                        <button
                          className="px-3 py-1 bg-gray-600 rounded text-sm cursor-not-allowed"
                          disabled
                        >
                          Installed
                        </button>
                      ) : (
                        <button
                          onClick={() => installPlugin(plugin.id)}
                          disabled={installing}
                          className={`px-3 py-1 rounded text-sm ${
                            installing
                              ? 'bg-gray-600 cursor-wait'
                              : 'bg-blue-600 hover:bg-blue-700'
                          }`}
                        >
                          {installing ? 'Installing...' : 'Install'}
                        </button>
                      )}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default PluginManager;
