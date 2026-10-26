// Preserve the existing rule's file-level shadow check at low.
import fetch from 'local-fetch';
fetch('/local', { cache: 'force-cache' });
