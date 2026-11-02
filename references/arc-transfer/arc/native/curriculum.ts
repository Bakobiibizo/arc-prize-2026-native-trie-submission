/** Explicit symbolic curriculum. No generated grid-answer examples. */
import type {SqliteTrieStore} from '../../src/storage.ts';
import {teach,variable,type Pattern,type Template} from './machine.ts';
const shape=(name:string,...children:Pattern[]):Pattern=>({kind:'structure',name,children});
const output=(name:string,...children:Template[]):Template=>({kind:'structure',name,children});
export function teachRows(store:SqliteTrieStore) {
  const a=variable('a'),b=variable('b'),q=variable('q'),r=variable('r');
  teach(store,'swap',shape('grid',a,b),output('grid',b,a),'curriculum:swap');
  teach(store,'repeat',shape('grid',a,b),output('grid',a,b,a,b),'curriculum:repeat');
  teach(store,'identity',shape('grid',a,b),output('grid',a,b),'curriculum:identity');
  // Demonstration relation is encoded by repeated identities in its structural
  // address. This selects a continuation; no host compares candidate operators.
  teach(store,'solve',shape('example-query',shape('grid',a,b),shape('grid',b,a),shape('grid',q,r)),
    {kind:'call',name:'swap',argument:output('grid',q,r)},'curriculum:solve-swap');
  teach(store,'solve',shape('example-query',shape('grid',a,b),shape('grid',a,b),shape('grid',q,r)),
    {kind:'call',name:'identity',argument:output('grid',q,r)},'curriculum:solve-identity');
}
