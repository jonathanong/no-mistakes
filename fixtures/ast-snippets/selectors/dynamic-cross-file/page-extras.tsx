import DefaultFn from './extras';
import DefaultObj from './default-obj';
import DefaultArrowExpr from './default-arrow-expr';
import DefaultArrowBlock from './default-arrow-block';
import DefaultLiteral from './default-literal';
import DefaultObjComputed from './default-obj-computed';
import {
  fnWithExprConsequent,
  fnBareReturn,
  fnBlockBody,
  uninitializedLet,
  firstArr,
} from './extras';
import * as Ns from './selectors';

export function Page() {
  return (
    <div
      data-fn={DefaultFn}
      data-obj={DefaultObj}
      data-arrow-expr={DefaultArrowExpr}
      data-arrow-block={DefaultArrowBlock}
      data-literal={DefaultLiteral}
      data-expr-fn={fnWithExprConsequent}
      data-bare-return={fnBareReturn}
      data-block-body={fnBlockBody}
      data-obj-computed={DefaultObjComputed}
      data-uninit={uninitializedLet}
      data-arr={firstArr}
      data-ns={Ns}
    />
  );
}
