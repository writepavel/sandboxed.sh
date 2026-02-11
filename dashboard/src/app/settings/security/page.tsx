'use client';

import { useState } from 'react';
import useSWR from 'swr';
import { toast } from '@/components/toast';
import { getAuthStatus, changePassword } from '@/lib/api';
import { Shield, Lock, Clock, AlertTriangle, Info } from 'lucide-react';

export default function SecuritySettingsPage() {
  const { data: authStatus, isLoading, mutate } = useSWR(
    'auth-status',
    getAuthStatus,
    { revalidateOnFocus: false }
  );

  // Change password form state
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [saving, setSaving] = useState(false);

  const isMultiUser = authStatus?.auth_mode === 'multi_user';
  const hasExistingPassword = authStatus?.password_source !== 'none';

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();

    if (newPassword !== confirmPassword) {
      toast.error('Passwords do not match');
      return;
    }

    if (newPassword.length < 8) {
      toast.error('Password must be at least 8 characters');
      return;
    }

    setSaving(true);
    try {
      await changePassword({
        current_password: hasExistingPassword ? currentPassword : undefined,
        new_password: newPassword,
      });

      toast.success('Password updated successfully');
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      mutate();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setSaving(false);
    }
  };

  const passwordSourceLabel = {
    dashboard: 'Dashboard-managed',
    environment: 'Environment variable',
    none: 'Not configured',
  }[authStatus?.password_source ?? 'none'];

  const authModeLabel = {
    disabled: 'Disabled',
    single_tenant: 'Single Tenant',
    multi_user: 'Multi-User',
  }[authStatus?.auth_mode ?? 'disabled'];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-white">Security</h1>
        <p className="mt-1 text-sm text-white/50">
          Authentication status and password management
        </p>
      </div>

      {/* Auth Status Card */}
      <div className="rounded-xl border border-white/[0.06] bg-white/[0.02] p-6">
        <div className="flex items-center gap-3 mb-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.06]">
            <Shield className="h-5 w-5 text-white/70" />
          </div>
          <div>
            <h2 className="text-base font-medium text-white">Authentication Status</h2>
            <p className="text-xs text-white/40">Current authentication configuration</p>
          </div>
        </div>

        {isLoading ? (
          <div className="animate-pulse space-y-3">
            <div className="h-4 w-48 rounded bg-white/[0.06]" />
            <div className="h-4 w-36 rounded bg-white/[0.06]" />
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="rounded-lg bg-white/[0.03] border border-white/[0.04] p-4">
              <p className="text-xs text-white/40 mb-1">Auth Mode</p>
              <span className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${
                authStatus?.auth_mode === 'disabled'
                  ? 'bg-yellow-500/10 text-yellow-400'
                  : authStatus?.auth_mode === 'multi_user'
                  ? 'bg-blue-500/10 text-blue-400'
                  : 'bg-emerald-500/10 text-emerald-400'
              }`}>
                {authModeLabel}
              </span>
            </div>

            <div className="rounded-lg bg-white/[0.03] border border-white/[0.04] p-4">
              <p className="text-xs text-white/40 mb-1">Password Source</p>
              <span className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${
                authStatus?.password_source === 'dashboard'
                  ? 'bg-emerald-500/10 text-emerald-400'
                  : authStatus?.password_source === 'environment'
                  ? 'bg-blue-500/10 text-blue-400'
                  : 'bg-white/[0.06] text-white/50'
              }`}>
                {passwordSourceLabel}
              </span>
            </div>

            {authStatus?.password_changed_at && (
              <div className="rounded-lg bg-white/[0.03] border border-white/[0.04] p-4">
                <p className="text-xs text-white/40 mb-1">Last Changed</p>
                <p className="flex items-center gap-1.5 text-sm text-white/70">
                  <Clock className="h-3.5 w-3.5" />
                  {new Date(authStatus.password_changed_at).toLocaleString()}
                </p>
              </div>
            )}

            {authStatus?.dev_mode && (
              <div className="rounded-lg bg-yellow-500/5 border border-yellow-500/10 p-4">
                <p className="flex items-center gap-1.5 text-xs text-yellow-400">
                  <AlertTriangle className="h-3.5 w-3.5" />
                  Dev mode is enabled &mdash; authentication is bypassed
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Change Password Card */}
      <div className="rounded-xl border border-white/[0.06] bg-white/[0.02] p-6">
        <div className="flex items-center gap-3 mb-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.06]">
            <Lock className="h-5 w-5 text-white/70" />
          </div>
          <div>
            <h2 className="text-base font-medium text-white">
              {hasExistingPassword ? 'Change Password' : 'Set Password'}
            </h2>
            <p className="text-xs text-white/40">
              {hasExistingPassword
                ? 'Update your dashboard login password'
                : 'Configure a dashboard login password'}
            </p>
          </div>
        </div>

        {isMultiUser ? (
          <div className="rounded-lg bg-white/[0.03] border border-white/[0.04] p-4">
            <p className="flex items-center gap-2 text-sm text-white/50">
              <Info className="h-4 w-4 shrink-0" />
              In multi-user mode, passwords are managed via the <code className="mx-1 rounded bg-white/[0.06] px-1.5 py-0.5 text-xs font-mono">SANDBOXED_USERS</code> environment variable.
            </p>
          </div>
        ) : (
          <form onSubmit={handleChangePassword} className="space-y-4 max-w-md">
            {!hasExistingPassword && (
              <div className="rounded-lg bg-blue-500/5 border border-blue-500/10 p-3">
                <p className="flex items-center gap-2 text-xs text-blue-400">
                  <Info className="h-3.5 w-3.5 shrink-0" />
                  No password is configured. Set one to enable authentication.
                  {!authStatus?.dev_mode && ' You will also need JWT_SECRET set as an environment variable.'}
                </p>
              </div>
            )}

            {hasExistingPassword && (
              <div>
                <label className="block text-sm font-medium text-white/70 mb-1.5">
                  Current Password
                </label>
                <input
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  className="w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder-white/30 focus:border-white/20 focus:outline-none focus:ring-1 focus:ring-white/20"
                  placeholder="Enter current password"
                  required
                />
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-white/70 mb-1.5">
                New Password
              </label>
              <input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                className="w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder-white/30 focus:border-white/20 focus:outline-none focus:ring-1 focus:ring-white/20"
                placeholder="At least 8 characters"
                minLength={8}
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-white/70 mb-1.5">
                Confirm New Password
              </label>
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder-white/30 focus:border-white/20 focus:outline-none focus:ring-1 focus:ring-white/20"
                placeholder="Confirm new password"
                minLength={8}
                required
              />
              {confirmPassword && newPassword !== confirmPassword && (
                <p className="mt-1 text-xs text-red-400">Passwords do not match</p>
              )}
            </div>

            <button
              type="submit"
              disabled={saving || !newPassword || newPassword !== confirmPassword || newPassword.length < 8}
              className="inline-flex items-center gap-2 rounded-lg bg-white/[0.08] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-white/[0.12] disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {saving ? (
                <>
                  <span className="h-4 w-4 animate-spin rounded-full border-2 border-white/20 border-t-white/70" />
                  Saving...
                </>
              ) : (
                <>
                  <Lock className="h-4 w-4" />
                  {hasExistingPassword ? 'Update Password' : 'Set Password'}
                </>
              )}
            </button>

            {authStatus?.password_source === 'environment' && (
              <p className="text-xs text-white/30">
                Setting a dashboard password will take priority over the DASHBOARD_PASSWORD environment variable.
              </p>
            )}
          </form>
        )}
      </div>
    </div>
  );
}
