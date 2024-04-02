import { browser } from '$app/environment';
import { writable } from 'svelte/store';

export const uname_store = writable((browser && localStorage.getItem('OCusername')) || '');
