import React, { useEffect, useState } from 'react';
import { api } from '../utils/api';
import './Dashboard.css';

const StatCard = ({ label, value, icon, color }) => (
    <div className="glass-panel stat-card">
        <div className="stat-info">
            <span className="stat-label">{label}</span>
            <span className="stat-value">{value}</span>
        </div>
        <div className="stat-icon" style={{ background: color + '20', color: color }}>
            {icon}
        </div>
    </div>
);

const Dashboard = () => {
    const [systemInfo, setSystemInfo] = useState(null);
    const [packages, setPackages] = useState([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchData = async () => {
            const [sys, pkgs] = await Promise.all([
                api.getSystemInfo(),
                api.getInstalledPackages()
            ]);
            setSystemInfo(sys);
            setPackages(pkgs);
            setLoading(false);
        };
        fetchData();
    }, []);

    if (loading) return <div className="loading">Loading system data...</div>;

    return (
        <div className="dashboard-page fade-in">
            <header className="page-header">
                <div>
                    <h1 className="page-title">System Overview</h1>
                    <p className="page-subtitle">Welcome back, {systemInfo?.hostname}</p>
                </div>
                <div className="os-badge glass-panel">
                    {systemInfo?.os} • {systemInfo?.arch}
                </div>
            </header>

            <div className="stats-grid">
                <StatCard
                    label="Installed Packages"
                    value={packages.length}
                    color="#3b82f6"
                    icon={<svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" fill="none" strokeWidth="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" /></svg>}
                />
                <StatCard
                    label="Pending Updates"
                    value="3"
                    color="#f59e0b"
                    icon={<svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" fill="none" strokeWidth="2"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" /></svg>}
                />
                <StatCard
                    label="System Health"
                    value="100%"
                    color="#10b981"
                    icon={<svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" fill="none" strokeWidth="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2" /></svg>}
                />
            </div>

            <div className="content-grid">
                <div className="glass-panel recent-activity">
                    <h3>Recent Installations</h3>
                    <div className="activity-list">
                        {packages.slice(0, 5).map((pkg, i) => (
                            <div key={i} className="activity-item">
                                <div className="box-icon">{pkg.box_type[0].toUpperCase()}</div>
                                <div className="activity-details">
                                    <span className="pkg-name">{pkg.name}</span>
                                    <span className="pkg-version">v{pkg.version}</span>
                                </div>
                                <span className="pkg-status">Installed</span>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="glass-panel system-status">
                    <h3>Quick Actions</h3>
                    <div className="action-grid">
                        <button className="action-btn">
                            <span>Update All</span>
                        </button>
                        <button className="action-btn">
                            <span>Clean Cache</span>
                        </button>
                        <button className="action-btn">
                            <span>System Scan</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Dashboard;
