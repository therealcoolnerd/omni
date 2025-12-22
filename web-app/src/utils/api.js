/**
 * API Service for Omni
 * Handles communication with the Rust backend
 * Falls back to mock data if backend is unreachable
 */

const API_BASE = 'http://localhost:3000/api';

const isBackendAvailable = async () => {
    try {
        await fetch(`${API_BASE}/system/info`, { method: 'HEAD' });
        return true;
    } catch (e) {
        return false;
    }
};

export const api = {
    getSystemInfo: async () => {
        try {
            const res = await fetch(`${API_BASE}/system/info`);
            if (!res.ok) throw new Error('Failed to fetch');
            return await res.json();
        } catch (e) {
            console.warn('Backend unavailable, using mock data');
            return {
                os: 'Windows 11',
                arch: 'x86_64',
                hostname: 'DESKTOP-OMNI'
            };
        }
    },

    getInstalledPackages: async () => {
        try {
            const res = await fetch(`${API_BASE}/packages/installed`);
            if (!res.ok) throw new Error('Failed to fetch');
            return await res.json();
        } catch (e) {
            return [
                { name: 'rust', version: '1.75.0', box_type: 'winget', description: 'Systems programming language' },
                { name: 'git', version: '2.40.1', box_type: 'winget', description: 'Version control system' },
                { name: 'vscode', version: '1.85.0', box_type: 'winget', description: 'Code editor' },
                { name: 'nodejs', version: '20.10.0', box_type: 'winget', description: 'JavaScript runtime' },
                { name: 'docker', version: '24.0.7', box_type: 'winget', description: 'Container platform' },
            ];
        }
    },

    searchPackages: async (query) => {
        try {
            const res = await fetch(`${API_BASE}/packages/search?q=${encodeURIComponent(query)}`);
            if (!res.ok) throw new Error('Failed to fetch');
            return await res.json();
        } catch (e) {
            // Mock search
            return [
                { name: query, version: 'latest', box_type: 'winget', description: `Result for ${query}` },
                { name: `${query}-cli`, version: '1.0.0', box_type: 'winget', description: `CLI tool for ${query}` },
            ];
        }
    }
};
