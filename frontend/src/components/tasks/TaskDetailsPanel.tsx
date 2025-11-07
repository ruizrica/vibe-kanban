import { useEffect, useState } from 'react';
import TaskDetailsHeader from './TaskDetailsHeader';
import { TaskFollowUpSection } from './TaskFollowUpSection';
import { EditorSelectionDialog } from './EditorSelectionDialog';
import {
  getBackdropClasses,
  getTaskPanelClasses,
} from '@/lib/responsive-config';
import type { TaskWithAttemptStatus } from 'shared/types';
import DiffTab from '@/components/tasks/TaskDetails/DiffTab.tsx';
import LogsTab from '@/components/tasks/TaskDetails/LogsTab.tsx';
import SpecTab from '@/components/tasks/TaskDetails/SpecTab.tsx';
import DeleteFileConfirmationDialog from '@/components/tasks/DeleteFileConfirmationDialog.tsx';
import TabNavigation from '@/components/tasks/TaskDetails/TabNavigation.tsx';
import CollapsibleToolbar from '@/components/tasks/TaskDetails/CollapsibleToolbar.tsx';
import TaskDetailsProvider from '../context/TaskDetailsContextProvider.tsx';
import { specsApi } from '@/lib/api';

interface TaskDetailsPanelProps {
  task: TaskWithAttemptStatus | null;
  projectHasDevScript?: boolean;
  projectId: string;
  onClose: () => void;
  onEditTask?: (task: TaskWithAttemptStatus) => void;
  onDeleteTask?: (taskId: string) => void;
  isDialogOpen?: boolean;
}

export function TaskDetailsPanel({
  task,
  projectHasDevScript,
  projectId,
  onClose,
  onEditTask,
  onDeleteTask,
  isDialogOpen = false,
}: TaskDetailsPanelProps) {
  const [showEditorDialog, setShowEditorDialog] = useState(false);
  const [hasSpec, setHasSpec] = useState(false);

  // Tab and collapsible state
  const [activeTab, setActiveTab] = useState<'logs' | 'diffs' | 'spec'>('logs');
  const [userSelectedTab, setUserSelectedTab] = useState<boolean>(false);

  // Check if task has a spec
  useEffect(() => {
    if (!task?.id) {
      setHasSpec(false);
      return;
    }

    const taskId = task.id;
    let cancelled = false;

    (async () => {
      try {
        await specsApi.get(projectId, taskId);
        if (!cancelled) setHasSpec(true);
      } catch {
        if (!cancelled) setHasSpec(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [task?.id, projectId]);

  // Reset to spec tab (if available) or logs tab when task changes
  useEffect(() => {
    if (task?.id) {
      // Default to spec tab if it exists, otherwise logs
      setActiveTab(hasSpec ? 'spec' : 'logs');
      setUserSelectedTab(true); // Treat this as a user selection to prevent auto-switching
    }
  }, [task?.id, hasSpec]);

  // Handle ESC key locally to prevent global navigation
  useEffect(() => {
    if (isDialogOpen) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        onClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown, true);
    return () => document.removeEventListener('keydown', handleKeyDown, true);
  }, [onClose, isDialogOpen]);

  return (
    <>
      {!task ? null : (
        <TaskDetailsProvider
          task={task}
          projectId={projectId}
          setShowEditorDialog={setShowEditorDialog}
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          userSelectedTab={userSelectedTab}
          projectHasDevScript={projectHasDevScript}
        >
          {/* Backdrop - only on smaller screens (overlay mode) */}
          <div className={getBackdropClasses()} onClick={onClose} />

          {/* Panel */}
          <div className={getTaskPanelClasses()}>
            <div className="flex flex-col h-full">
              <TaskDetailsHeader
                onClose={onClose}
                onEditTask={onEditTask}
                onDeleteTask={onDeleteTask}
              />

              <CollapsibleToolbar />

              <TabNavigation
                activeTab={activeTab}
                setActiveTab={setActiveTab}
                setUserSelectedTab={setUserSelectedTab}
                hasSpec={hasSpec}
              />

              {/* Tab Content */}
              <div
                className={`flex-1 flex flex-col min-h-0 ${activeTab === 'logs' ? 'p-4' : activeTab === 'spec' ? '' : 'pt-4'}`}
              >
                {activeTab === 'diffs' ? <DiffTab /> : activeTab === 'spec' ? <SpecTab /> : <LogsTab />}
              </div>

              <TaskFollowUpSection />
            </div>
          </div>

          <EditorSelectionDialog
            isOpen={showEditorDialog}
            onClose={() => setShowEditorDialog(false)}
          />

          <DeleteFileConfirmationDialog />
        </TaskDetailsProvider>
      )}
    </>
  );
}
