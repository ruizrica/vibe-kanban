import { useContext } from 'react';
import { TaskDetailsContext } from '@/components/context/taskDetailsContext';
import { SpecViewer } from '../SpecViewer';

function SpecTab() {
  const { task, projectId } = useContext(TaskDetailsContext);

  if (!task) {
    return null;
  }

  return (
    <div className="overflow-y-auto h-full">
      <SpecViewer
        projectId={projectId}
        taskId={task.id}
      />
    </div>
  );
}

export default SpecTab;
