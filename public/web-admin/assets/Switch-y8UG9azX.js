import{O as e,T as t,_ as n,at as r,lt as i}from"./use-message-CAsKU2HG.js";import{$ as a,A as o,An as s,D as c,Gt as l,Mn as u,N as d,Nn as f,Pn as p,S as m,Sn as h,Tn as g,_t as _,et as v,jn as y,kn as b,lt as x,nn as S,ot as C,tt as w,w as T}from"./Popconfirm-DhaSoTKv.js";var E={buttonHeightSmall:`14px`,buttonHeightMedium:`18px`,buttonHeightLarge:`22px`,buttonWidthSmall:`14px`,buttonWidthMedium:`18px`,buttonWidthLarge:`22px`,buttonWidthPressedSmall:`20px`,buttonWidthPressedMedium:`24px`,buttonWidthPressedLarge:`28px`,railHeightSmall:`18px`,railHeightMedium:`22px`,railHeightLarge:`26px`,railWidthSmall:`32px`,railWidthMedium:`40px`,railWidthLarge:`48px`};function D(e){let{primaryColor:t,opacityDisabled:n,borderRadius:r,textColor3:i}=e;return Object.assign(Object.assign({},E),{iconColor:i,textColor:`white`,loadingColor:t,opacityDisabled:n,railColor:`rgba(0, 0, 0, .14)`,railColorActive:t,buttonBoxShadow:`0 1px 4px 0 rgba(0, 0, 0, 0.3), inset 0 0 1px 0 rgba(0, 0, 0, 0.05)`,buttonColor:`#FFF`,railBorderRadiusSmall:r,railBorderRadiusMedium:r,railBorderRadiusLarge:r,buttonBorderRadiusSmall:r,buttonBorderRadiusMedium:r,buttonBorderRadiusLarge:r,boxShadowFocus:`0 0 0 2px ${S(t,{alpha:.2})}`})}var O={name:`Switch`,common:m,self:D},k=s(`switch`,`
 height: var(--n-height);
 min-width: var(--n-width);
 vertical-align: middle;
 user-select: none;
 -webkit-user-select: none;
 display: inline-flex;
 outline: none;
 justify-content: center;
 align-items: center;
`,[y(`children-placeholder`,`
 height: var(--n-rail-height);
 display: flex;
 flex-direction: column;
 overflow: hidden;
 pointer-events: none;
 visibility: hidden;
 `),y(`rail-placeholder`,`
 display: flex;
 flex-wrap: none;
 `),y(`button-placeholder`,`
 width: calc(1.75 * var(--n-rail-height));
 height: var(--n-rail-height);
 `),s(`base-loading`,`
 position: absolute;
 top: 50%;
 left: 50%;
 transform: translateX(-50%) translateY(-50%);
 font-size: calc(var(--n-button-width) - 4px);
 color: var(--n-loading-color);
 transition: color .3s var(--n-bezier);
 `,[c({left:`50%`,top:`50%`,originalTransform:`translateX(-50%) translateY(-50%)`})]),y(`checked, unchecked`,`
 transition: color .3s var(--n-bezier);
 color: var(--n-text-color);
 box-sizing: border-box;
 position: absolute;
 white-space: nowrap;
 top: 0;
 bottom: 0;
 display: flex;
 align-items: center;
 line-height: 1;
 `),y(`checked`,`
 right: 0;
 padding-right: calc(1.25 * var(--n-rail-height) - var(--n-offset));
 `),y(`unchecked`,`
 left: 0;
 justify-content: flex-end;
 padding-left: calc(1.25 * var(--n-rail-height) - var(--n-offset));
 `),b(`&:focus`,[y(`rail`,`
 box-shadow: var(--n-box-shadow-focus);
 `)]),u(`round`,[y(`rail`,`border-radius: calc(var(--n-rail-height) / 2);`,[y(`button`,`border-radius: calc(var(--n-button-height) / 2);`)])]),f(`disabled`,[f(`icon`,[u(`rubber-band`,[u(`pressed`,[y(`rail`,[y(`button`,`max-width: var(--n-button-width-pressed);`)])]),y(`rail`,[b(`&:active`,[y(`button`,`max-width: var(--n-button-width-pressed);`)])]),u(`active`,[u(`pressed`,[y(`rail`,[y(`button`,`left: calc(100% - var(--n-offset) - var(--n-button-width-pressed));`)])]),y(`rail`,[b(`&:active`,[y(`button`,`left: calc(100% - var(--n-offset) - var(--n-button-width-pressed));`)])])])])])]),u(`active`,[y(`rail`,[y(`button`,`left: calc(100% - var(--n-button-width) - var(--n-offset))`)])]),y(`rail`,`
 overflow: hidden;
 height: var(--n-rail-height);
 min-width: var(--n-rail-width);
 border-radius: var(--n-rail-border-radius);
 cursor: pointer;
 position: relative;
 transition:
 opacity .3s var(--n-bezier),
 background .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier);
 background-color: var(--n-rail-color);
 `,[y(`button-icon`,`
 color: var(--n-icon-color);
 transition: color .3s var(--n-bezier);
 font-size: calc(var(--n-button-height) - 4px);
 position: absolute;
 left: 0;
 right: 0;
 top: 0;
 bottom: 0;
 display: flex;
 justify-content: center;
 align-items: center;
 line-height: 1;
 `,[c()]),y(`button`,`
 align-items: center; 
 top: var(--n-offset);
 left: var(--n-offset);
 height: var(--n-button-height);
 width: var(--n-button-width-pressed);
 max-width: var(--n-button-width);
 border-radius: var(--n-button-border-radius);
 background-color: var(--n-button-color);
 box-shadow: var(--n-button-box-shadow);
 box-sizing: border-box;
 cursor: inherit;
 content: "";
 position: absolute;
 transition:
 background-color .3s var(--n-bezier),
 left .3s var(--n-bezier),
 opacity .3s var(--n-bezier),
 max-width .3s var(--n-bezier),
 box-shadow .3s var(--n-bezier);
 `)]),u(`active`,[y(`rail`,`background-color: var(--n-rail-color-active);`)]),u(`loading`,[y(`rail`,`
 cursor: wait;
 `)]),u(`disabled`,[y(`rail`,`
 cursor: not-allowed;
 opacity: .5;
 `)])]),A=Object.assign(Object.assign({},d.props),{size:String,value:{type:[String,Number,Boolean],default:void 0},loading:Boolean,defaultValue:{type:[String,Number,Boolean],default:!1},disabled:{type:Boolean,default:void 0},round:{type:Boolean,default:!0},"onUpdate:value":[Function,Array],onUpdateValue:[Function,Array],checkedValue:{type:[String,Number,Boolean],default:!0},uncheckedValue:{type:[String,Number,Boolean],default:!1},railStyle:Function,rubberBand:{type:Boolean,default:!0},spinProps:Object,onChange:[Function,Array]}),j,M=t({name:`Switch`,props:A,slots:Object,setup(e){j===void 0&&(j=typeof CSS<`u`?CSS.supports!==void 0&&CSS.supports(`width`,`max(1px)`):!0);let{mergedClsPrefixRef:t,inlineThemeDisabled:o,mergedComponentPropsRef:s}=w(e),c=d(`Switch`,`-switch`,k,O,e,t),u=a(e,{mergedSize(t){return e.size===void 0?t?t.mergedSize.value:s?.value?.Switch?.size||`medium`:e.size}}),{mergedSizeRef:f,mergedDisabledRef:m}=u,y=r(e.defaultValue),b=l(i(e,`value`),y),x=n(()=>b.value===e.checkedValue),S=r(!1),C=r(!1),T=n(()=>{let{railStyle:t}=e;if(t)return t({focused:C.value,checked:x.value})});function E(t){let{"onUpdate:value":n,onChange:r,onUpdateValue:i}=e,{nTriggerFormInput:a,nTriggerFormChange:o}=u;n&&_(n,t),i&&_(i,t),r&&_(r,t),y.value=t,a(),o()}function D(){let{nTriggerFormFocus:e}=u;e()}function A(){let{nTriggerFormBlur:e}=u;e()}function M(){e.loading||m.value||(b.value===e.checkedValue?E(e.uncheckedValue):E(e.checkedValue))}function N(){C.value=!0,D()}function P(){C.value=!1,A(),S.value=!1}function F(t){e.loading||m.value||t.key===` `&&(b.value===e.checkedValue?E(e.uncheckedValue):E(e.checkedValue),S.value=!1)}function I(t){e.loading||m.value||t.key===` `&&(t.preventDefault(),S.value=!0)}let L=n(()=>{let{value:e}=f,{self:{opacityDisabled:t,railColor:n,railColorActive:r,buttonBoxShadow:i,buttonColor:a,boxShadowFocus:o,loadingColor:s,textColor:l,iconColor:u,[p(`buttonHeight`,e)]:d,[p(`buttonWidth`,e)]:m,[p(`buttonWidthPressed`,e)]:_,[p(`railHeight`,e)]:v,[p(`railWidth`,e)]:y,[p(`railBorderRadius`,e)]:b,[p(`buttonBorderRadius`,e)]:x},common:{cubicBezierEaseInOut:S}}=c.value,C,w,T;return j?(C=`calc((${v} - ${d}) / 2)`,w=`max(${v}, ${d})`,T=`max(${y}, calc(${y} + ${d} - ${v}))`):(C=g((h(v)-h(d))/2),w=g(Math.max(h(v),h(d))),T=h(v)>h(d)?y:g(h(y)+h(d)-h(v))),{"--n-bezier":S,"--n-button-border-radius":x,"--n-button-box-shadow":i,"--n-button-color":a,"--n-button-width":m,"--n-button-width-pressed":_,"--n-button-height":d,"--n-height":w,"--n-offset":C,"--n-opacity-disabled":t,"--n-rail-border-radius":b,"--n-rail-color":n,"--n-rail-color-active":r,"--n-rail-height":v,"--n-rail-width":y,"--n-width":T,"--n-box-shadow-focus":o,"--n-loading-color":s,"--n-text-color":l,"--n-icon-color":u}}),R=o?v(`switch`,n(()=>f.value[0]),L,e):void 0;return{handleClick:M,handleBlur:P,handleFocus:N,handleKeyup:F,handleKeydown:I,mergedRailStyle:T,pressed:S,mergedClsPrefix:t,mergedValue:b,checked:x,mergedDisabled:m,cssVars:o?void 0:L,themeClass:R?.themeClass,onRender:R?.onRender}},render(){let{mergedClsPrefix:t,mergedDisabled:n,checked:r,mergedRailStyle:i,onRender:a,$slots:s}=this;a?.();let{checked:c,unchecked:l,icon:u,"checked-icon":d,"unchecked-icon":f}=s,p=!(C(u)&&C(d)&&C(f));return e(`div`,{role:`switch`,"aria-checked":r,class:[`${t}-switch`,this.themeClass,p&&`${t}-switch--icon`,r&&`${t}-switch--active`,n&&`${t}-switch--disabled`,this.round&&`${t}-switch--round`,this.loading&&`${t}-switch--loading`,this.pressed&&`${t}-switch--pressed`,this.rubberBand&&`${t}-switch--rubber-band`],tabindex:this.mergedDisabled?void 0:0,style:this.cssVars,onClick:this.handleClick,onFocus:this.handleFocus,onBlur:this.handleBlur,onKeyup:this.handleKeyup,onKeydown:this.handleKeydown},e(`div`,{class:`${t}-switch__rail`,"aria-hidden":`true`,style:i},x(c,n=>x(l,r=>n||r?e(`div`,{"aria-hidden":!0,class:`${t}-switch__children-placeholder`},e(`div`,{class:`${t}-switch__rail-placeholder`},e(`div`,{class:`${t}-switch__button-placeholder`}),n),e(`div`,{class:`${t}-switch__rail-placeholder`},e(`div`,{class:`${t}-switch__button-placeholder`}),r)):null)),e(`div`,{class:`${t}-switch__button`},x(u,n=>x(d,r=>x(f,i=>e(o,null,{default:()=>this.loading?e(T,Object.assign({key:`loading`,clsPrefix:t,strokeWidth:20},this.spinProps)):this.checked&&(r||n)?e(`div`,{class:`${t}-switch__button-icon`,key:r?`checked-icon`:`icon`},r||n):!this.checked&&(i||n)?e(`div`,{class:`${t}-switch__button-icon`,key:i?`unchecked-icon`:`icon`},i||n):null})))),x(c,n=>n&&e(`div`,{key:`checked`,class:`${t}-switch__checked`},n)),x(l,n=>n&&e(`div`,{key:`unchecked`,class:`${t}-switch__unchecked`},n)))))}});export{A as n,M as t};