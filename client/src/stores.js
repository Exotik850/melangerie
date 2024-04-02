import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import { checkUser } from '$lib/index';

function get_uname() {
  if (localStorage.getItem('OCusername')) {
    if (checkUser(localStorage.getItem('OCusername'))) {
      return localStorage.getItem('OCusername');
    }
  }
}

export const uname_store = writable((browser && get_uname()) || '');