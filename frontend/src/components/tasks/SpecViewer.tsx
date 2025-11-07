import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { CheckCircle, XCircle, FileEdit } from 'lucide-react';
import { specsApi } from '@/lib/api';
import { Loader } from '@/components/ui/loader';
import type { TaskSpec, TaskSpecStatus, UserStory } from 'shared/types';

interface SpecViewerProps {
  projectId: string;
  taskId: string;
  onEdit?: () => void;
  onApprove?: () => void;
}

const STATUS_COLORS: Record<TaskSpecStatus, string> = {
  draft: 'bg-gray-500',
  review: 'bg-blue-500',
  approved: 'bg-green-500',
  rejected: 'bg-red-500',
};

const STATUS_LABELS: Record<TaskSpecStatus, string> = {
  draft: 'Draft',
  review: 'In Review',
  approved: 'Approved',
  rejected: 'Rejected',
};

export function SpecViewer({
  projectId,
  taskId,
  onEdit,
  onApprove,
}: SpecViewerProps) {
  const [spec, setSpec] = useState<TaskSpec | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isApproving, setIsApproving] = useState(false);
  const [isRejecting, setIsRejecting] = useState(false);
  const [rejectionReason, setRejectionReason] = useState('');
  const [showRejectionInput, setShowRejectionInput] = useState(false);

  useEffect(() => {
    loadSpec();
  }, [projectId, taskId]);

  const loadSpec = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await specsApi.get(projectId, taskId);
      setSpec(data);
    } catch (err: any) {
      if (err?.status === 404) {
        setError('No specification found for this task');
      } else {
        setError('Failed to load specification');
      }
    } finally {
      setLoading(false);
    }
  };

  const handleApprove = async () => {
    if (!spec) return;

    setIsApproving(true);
    try {
      const updated = await specsApi.updateStatus(projectId, taskId, {
        status: 'approved',
        approved_by: 'user', // TODO: Get from auth context
        rejection_reason: null,
      });
      setSpec(updated);
      onApprove?.();
    } catch (err) {
      setError('Failed to approve specification');
    } finally {
      setIsApproving(false);
    }
  };

  const handleReject = async () => {
    if (!spec || !rejectionReason.trim()) return;

    setIsRejecting(true);
    try {
      const updated = await specsApi.updateStatus(projectId, taskId, {
        status: 'rejected',
        approved_by: null,
        rejection_reason: rejectionReason,
      });
      setSpec(updated);
      setShowRejectionInput(false);
      setRejectionReason('');
    } catch (err) {
      setError('Failed to reject specification');
    } finally {
      setIsRejecting(false);
    }
  };

  const parseUserStories = (jsonStr: string): UserStory[] => {
    try {
      return JSON.parse(jsonStr);
    } catch {
      return [];
    }
  };

  const parseScope = (jsonStr: string): string[] => {
    try {
      return JSON.parse(jsonStr);
    } catch {
      return [];
    }
  };

  if (loading) {
    return (
      <div className="p-6 flex justify-center">
        <Loader />
      </div>
    );
  }

  if (error && !spec) {
    return (
      <div className="p-6">
        <div className="text-center text-muted-foreground">{error}</div>
      </div>
    );
  }

  if (!spec) {
    return null;
  }

  const userStories = parseUserStories(spec.user_stories);
  const scopeItems = parseScope(spec.scope);

  return (
    <div className="p-6 space-y-6">
      {/* Header with status */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Task Specification</h2>
        <Badge className={STATUS_COLORS[spec.status]}>
          {STATUS_LABELS[spec.status]}
        </Badge>
      </div>

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3 text-sm text-red-800 dark:text-red-200">
          {error}
        </div>
      )}

      {/* Overview */}
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-2">
          Overview
        </h3>
        <p className="text-sm">{spec.overview}</p>
      </div>

      {/* User Stories */}
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-2">
          User Stories
        </h3>
        <div className="space-y-3">
          {userStories.map((story, index) => (
            <div
              key={index}
              className="border rounded-lg p-3 space-y-2 bg-muted/30"
            >
              <div>
                <span className="text-xs font-medium text-muted-foreground">
                  Workflow:
                </span>
                <p className="text-sm mt-1">{story.workflow}</p>
              </div>
              <div>
                <span className="text-xs font-medium text-muted-foreground">
                  Problem Solved:
                </span>
                <p className="text-sm mt-1">{story.problem_solved}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Scope */}
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-2">
          Scope
        </h3>
        <ul className="space-y-1">
          {scopeItems.map((item, index) => (
            <li key={index} className="text-sm flex items-start gap-2">
              <span className="text-muted-foreground mt-1">•</span>
              <span>{item}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Out of Scope */}
      {spec.out_of_scope && (
        <div>
          <h3 className="text-sm font-medium text-muted-foreground mb-2">
            Out of Scope
          </h3>
          <p className="text-sm">{spec.out_of_scope}</p>
        </div>
      )}

      {/* Deliverables */}
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-2">
          Expected Deliverable
        </h3>
        <p className="text-sm">{spec.deliverables}</p>
      </div>

      {/* Rejection Reason (if rejected) */}
      {spec.status === 'rejected' && spec.rejection_reason && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <h3 className="text-sm font-medium text-red-900 dark:text-red-100 mb-2">
            Rejection Reason
          </h3>
          <p className="text-sm text-red-800 dark:text-red-200">
            {spec.rejection_reason}
          </p>
        </div>
      )}

      {/* Approval Info */}
      {spec.status === 'approved' && spec.approved_by && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
          <p className="text-sm text-green-800 dark:text-green-200">
            Approved by {spec.approved_by}
            {spec.approved_at && (
              <span className="text-muted-foreground">
                {' '}
                on {new Date(spec.approved_at).toLocaleString()}
              </span>
            )}
          </p>
        </div>
      )}

      {/* Action Buttons */}
      {spec.status !== 'approved' ? (
        <div className="space-y-3 pt-4 border-t">
          {!showRejectionInput ? (
            <div className="flex gap-2">
              {onEdit && (
                <Button
                  variant="outline"
                  onClick={onEdit}
                  disabled={isApproving || isRejecting}
                >
                  <FileEdit className="h-4 w-4 mr-2" />
                  Edit
                </Button>
              )}
              {spec.status === 'review' && (
                <>
                  <Button
                    variant="outline"
                    onClick={() => setShowRejectionInput(true)}
                    disabled={isApproving || isRejecting}
                    className="text-red-600 hover:text-red-700"
                  >
                    <XCircle className="h-4 w-4 mr-2" />
                    Reject
                  </Button>
                  <Button
                    onClick={handleApprove}
                    disabled={isApproving || isRejecting}
                    className="bg-green-600 hover:bg-green-700"
                  >
                    {isApproving ? (
                      'Approving...'
                    ) : (
                      <>
                        <CheckCircle className="h-4 w-4 mr-2" />
                        Approve
                      </>
                    )}
                  </Button>
                </>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              <div>
                <Label htmlFor="rejection-reason" className="text-sm font-medium">
                  Rejection Reason
                </Label>
                <Textarea
                  id="rejection-reason"
                  value={rejectionReason}
                  onChange={(e) => setRejectionReason(e.target.value)}
                  placeholder="Explain why this specification is being rejected..."
                  className="mt-1.5"
                  rows={3}
                  disabled={isRejecting}
                />
              </div>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  onClick={() => {
                    setShowRejectionInput(false);
                    setRejectionReason('');
                  }}
                  disabled={isRejecting}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleReject}
                  disabled={isRejecting || !rejectionReason.trim()}
                  className="bg-red-600 hover:bg-red-700"
                >
                  {isRejecting ? 'Rejecting...' : 'Confirm Rejection'}
                </Button>
              </div>
            </div>
          )}
        </div>
      ) : null}
    </div>
  );
}
