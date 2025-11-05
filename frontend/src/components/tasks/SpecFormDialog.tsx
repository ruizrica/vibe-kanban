import { useState, useEffect, useCallback } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import type { UserStory } from 'shared/types';

interface SpecFormDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  projectId: string;
  onCreateSpec: (spec: {
    title: string;
    overview: string;
    user_stories: UserStory[];
    scope: string[];
    out_of_scope: string | null;
    deliverables: string;
  }) => Promise<void>;
}

export function SpecFormDialog({
  isOpen,
  onOpenChange,
  projectId,
  onCreateSpec,
}: SpecFormDialogProps) {
  const [title, setTitle] = useState('');
  const [overview, setOverview] = useState('');
  const [userStories, setUserStories] = useState<UserStory[]>([
    { workflow: '', problem_solved: '' },
  ]);
  const [scope, setScope] = useState<string[]>(['']);
  const [outOfScope, setOutOfScope] = useState('');
  const [deliverables, setDeliverables] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      // Reset form when closed
      setTitle('');
      setOverview('');
      setUserStories([{ workflow: '', problem_solved: '' }]);
      setScope(['']);
      setOutOfScope('');
      setDeliverables('');
    }
  }, [isOpen]);

  const addUserStory = () => {
    if (userStories.length < 3) {
      setUserStories([...userStories, { workflow: '', problem_solved: '' }]);
    }
  };

  const removeUserStory = (index: number) => {
    if (userStories.length > 1) {
      setUserStories(userStories.filter((_, i) => i !== index));
    }
  };

  const updateUserStory = (
    index: number,
    field: 'workflow' | 'problem_solved',
    value: string
  ) => {
    const updated = [...userStories];
    updated[index][field] = value;
    setUserStories(updated);
  };

  const addScopeItem = () => {
    if (scope.length < 5) {
      setScope([...scope, '']);
    }
  };

  const removeScopeItem = (index: number) => {
    if (scope.length > 1) {
      setScope(scope.filter((_, i) => i !== index));
    }
  };

  const updateScopeItem = (index: number, value: string) => {
    const updated = [...scope];
    updated[index] = value;
    setScope(updated);
  };

  const handleSubmit = async () => {
    // Validate required fields
    if (!title.trim() || !overview.trim() || !deliverables.trim()) {
      return;
    }

    // Filter out empty user stories and scope items
    const validUserStories = userStories.filter(
      (story) => story.workflow.trim() && story.problem_solved.trim()
    );
    const validScope = scope.filter((item) => item.trim());

    if (validUserStories.length === 0 || validScope.length === 0) {
      return;
    }

    setIsSubmitting(true);
    try {
      await onCreateSpec({
        title,
        overview,
        user_stories: validUserStories,
        scope: validScope,
        out_of_scope: outOfScope.trim() || null,
        deliverables,
      });
      onOpenChange(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        handleCancel();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown, true);
      return () => document.removeEventListener('keydown', handleKeyDown, true);
    }
  }, [isOpen, handleCancel]);

  const isValid =
    title.trim() &&
    overview.trim() &&
    deliverables.trim() &&
    userStories.some((s) => s.workflow.trim() && s.problem_solved.trim()) &&
    scope.some((item) => item.trim());

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Task Specification (Agent OS)</DialogTitle>
        </DialogHeader>
        <div className="space-y-5">
          {/* Title */}
          <div>
            <Label htmlFor="spec-title" className="text-sm font-medium">
              Task Title
            </Label>
            <Input
              id="spec-title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Concise title for this task"
              className="mt-1.5"
              disabled={isSubmitting}
              autoFocus
            />
          </div>

          {/* Overview */}
          <div>
            <Label htmlFor="spec-overview" className="text-sm font-medium">
              Overview <span className="text-muted-foreground">(1-2 sentences)</span>
            </Label>
            <Textarea
              id="spec-overview"
              value={overview}
              onChange={(e) => setOverview(e.target.value)}
              placeholder="High-level summary of what this task accomplishes"
              className="mt-1.5"
              rows={2}
              disabled={isSubmitting}
            />
          </div>

          {/* User Stories */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <Label className="text-sm font-medium">
                User Stories <span className="text-muted-foreground">(1-3)</span>
              </Label>
              {userStories.length < 3 && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={addUserStory}
                  disabled={isSubmitting}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  Add Story
                </Button>
              )}
            </div>
            <div className="space-y-3">
              {userStories.map((story, index) => (
                <div key={index} className="border rounded-lg p-3 space-y-2">
                  <div className="flex items-center justify-between">
                    <Label className="text-xs font-medium text-muted-foreground">
                      Story {index + 1}
                    </Label>
                    {userStories.length > 1 && (
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => removeUserStory(index)}
                        disabled={isSubmitting}
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                  <Input
                    placeholder="Workflow (e.g., 'User clicks export button')"
                    value={story.workflow}
                    onChange={(e) =>
                      updateUserStory(index, 'workflow', e.target.value)
                    }
                    disabled={isSubmitting}
                  />
                  <Input
                    placeholder="Problem solved (e.g., 'User can download data as CSV')"
                    value={story.problem_solved}
                    onChange={(e) =>
                      updateUserStory(index, 'problem_solved', e.target.value)
                    }
                    disabled={isSubmitting}
                  />
                </div>
              ))}
            </div>
          </div>

          {/* Scope */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <Label className="text-sm font-medium">
                Scope <span className="text-muted-foreground">(1-5 features)</span>
              </Label>
              {scope.length < 5 && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={addScopeItem}
                  disabled={isSubmitting}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  Add Feature
                </Button>
              )}
            </div>
            <div className="space-y-2">
              {scope.map((item, index) => (
                <div key={index} className="flex gap-2">
                  <Input
                    placeholder={`Feature ${index + 1}`}
                    value={item}
                    onChange={(e) => updateScopeItem(index, e.target.value)}
                    disabled={isSubmitting}
                  />
                  {scope.length > 1 && (
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => removeScopeItem(index)}
                      disabled={isSubmitting}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Out of Scope */}
          <div>
            <Label htmlFor="spec-out-of-scope" className="text-sm font-medium">
              Out of Scope <span className="text-muted-foreground">(Optional)</span>
            </Label>
            <Textarea
              id="spec-out-of-scope"
              value={outOfScope}
              onChange={(e) => setOutOfScope(e.target.value)}
              placeholder="What is explicitly not included in this task"
              className="mt-1.5"
              rows={2}
              disabled={isSubmitting}
            />
          </div>

          {/* Deliverables */}
          <div>
            <Label htmlFor="spec-deliverables" className="text-sm font-medium">
              Expected Deliverable
            </Label>
            <Textarea
              id="spec-deliverables"
              value={deliverables}
              onChange={(e) => setDeliverables(e.target.value)}
              placeholder="What should be delivered when this task is complete"
              className="mt-1.5"
              rows={2}
              disabled={isSubmitting}
            />
          </div>

          {/* Buttons */}
          <div className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2 pt-2">
            <Button
              variant="outline"
              onClick={handleCancel}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={isSubmitting || !isValid}
              className="font-medium"
            >
              {isSubmitting ? 'Creating...' : 'Create Spec & Task'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
