import React from 'react';
import {
  Folder,
  FileCode,
  Atom,
  Binary,
  Coffee,
  Globe,
  Hexagon,
  Layers,
  Terminal,
  type LucideIcon,
} from 'lucide-react';

/**
 * 项目类型
 */
export type ProjectType =
  | 'react'
  | 'vue'
  | 'angular'
  | 'svelte'
  | 'nodejs'
  | 'nextjs'
  | 'python'
  | 'rust'
  | 'go'
  | 'java'
  | 'docker'
  | 'generic';

/**
 * 项目类型配置
 */
interface ProjectTypeConfig {
  icon: LucideIcon;
  color: string;
  bgColor: string;
  label: string;
}

const projectTypeConfigs: Record<ProjectType, ProjectTypeConfig> = {
  react: {
    icon: Atom,
    color: 'text-blue-400',
    bgColor: 'bg-blue-400/10',
    label: 'React',
  },
  vue: {
    icon: Hexagon,
    color: 'text-green-400',
    bgColor: 'bg-green-400/10',
    label: 'Vue',
  },
  angular: {
    icon: Layers,
    color: 'text-red-400',
    bgColor: 'bg-red-400/10',
    label: 'Angular',
  },
  svelte: {
    icon: Binary,
    color: 'text-orange-400',
    bgColor: 'bg-orange-400/10',
    label: 'Svelte',
  },
  nodejs: {
    icon: Terminal,
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    label: 'Node.js',
  },
  nextjs: {
    icon: Globe,
    color: 'text-white',
    bgColor: 'bg-white/10',
    label: 'Next.js',
  },
  python: {
    icon: Coffee,
    color: 'text-yellow-400',
    bgColor: 'bg-yellow-400/10',
    label: 'Python',
  },
  rust: {
    icon: Binary,
    color: 'text-orange-500',
    bgColor: 'bg-orange-500/10',
    label: 'Rust',
  },
  go: {
    icon: FileCode,
    color: 'text-cyan-400',
    bgColor: 'bg-cyan-400/10',
    label: 'Go',
  },
  java: {
    icon: Coffee,
    color: 'text-red-500',
    bgColor: 'bg-red-500/10',
    label: 'Java',
  },
  docker: {
    icon: Layers,
    color: 'text-blue-500',
    bgColor: 'bg-blue-500/10',
    label: 'Docker',
  },
  generic: {
    icon: Folder,
    color: 'text-gray-400',
    bgColor: 'bg-gray-400/10',
    label: '项目',
  },
};

/**
 * 根据项目路径检测项目类型
 *
 * 注意：这是一个简化版本，实际项目中可以通过读取项目文件来更准确地检测
 * 由于前端无法直接访问文件系统，这里使用基于路径的启发式检测
 */
export function detectProjectType(projectPath: string): ProjectType {
  if (!projectPath) return 'generic';

  const lowerPath = projectPath.toLowerCase();
  const pathParts = lowerPath.split(/[\/\\]/);

  // 基于路径关键词的启发式检测
  if (pathParts.some((p) => p.includes('react'))) return 'react';
  if (pathParts.some((p) => p.includes('vue'))) return 'vue';
  if (pathParts.some((p) => p.includes('angular'))) return 'angular';
  if (pathParts.some((p) => p.includes('svelte'))) return 'svelte';
  if (pathParts.some((p) => p.includes('next'))) return 'nextjs';
  if (pathParts.some((p) => p.includes('python') || p === 'py')) return 'python';
  if (pathParts.some((p) => p.includes('rust') || p === 'rs')) return 'rust';
  if (pathParts.some((p) => p.includes('go') || p === 'golang')) return 'go';
  if (pathParts.some((p) => p.includes('java'))) return 'java';
  if (pathParts.some((p) => p.includes('docker'))) return 'docker';

  return 'generic';
}

/**
 * 获取项目类型配置
 */
export function getProjectTypeConfig(
  projectType: ProjectType
): ProjectTypeConfig {
  return projectTypeConfigs[projectType] || projectTypeConfigs.generic;
}

/**
 * 项目图标组件 Props
 */
interface ProjectIconProps {
  projectPath: string;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

/**
 * 项目图标组件
 *
 * @example
 * <ProjectIcon projectPath="/Users/hejj/react-app" size="md" />
 * <ProjectIcon projectPath="/Users/hejj/python-project" size="lg" showLabel />
 */
export const ProjectIcon: React.FC<ProjectIconProps> = ({
  projectPath,
  size = 'md',
  showLabel = false,
  className,
}) => {
  const projectType = detectProjectType(projectPath);
  const config = getProjectTypeConfig(projectType);
  const Icon = config.icon;

  const sizeClasses = {
    sm: {
      container: 'w-8 h-8',
      icon: 'w-4 h-4',
    },
    md: {
      container: 'w-10 h-10',
      icon: 'w-5 h-5',
    },
    lg: {
      container: 'w-12 h-12',
      icon: 'w-6 h-6',
    },
  };

  const sizes = sizeClasses[size];

  return (
    <div
      className={[
        'flex items-center gap-2',
        className || '',
      ].join(' ')}
    >
      <div
        className={[
          sizes.container,
          'rounded-lg flex items-center justify-center',
          config.bgColor,
          className || '',
        ].join(' ')}
      >
        <Icon className={[sizes.icon, config.color].join(' ')} />
      </div>
      {showLabel && (
        <span className="text-sm text-text-secondary">{config.label}</span>
      )}
    </div>
  );
};

export default ProjectIcon;
