import dynamic from 'next/dynamic'

export const TopicEditTabs = dynamic(() => import('@lib/topic-edit-tabs'))
