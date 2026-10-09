"""Card/table animation for the Pineapple film; shared speech/publishing in render.py.

Timing cues are resolved from the actual TTS word boundaries, never from word counts.
The 720p artwork is drawn locally; no third-party images, music or footage.
"""
from functools import lru_cache
from PIL import Image, ImageDraw
import timeline

WIDTH, HEIGHT, SOURCE_FPS = 1280, 720, 15
BG, INK, GOLD, MUTED = '#0b1715', '#f7f1e3', '#e1c17a', '#b5c9c1'
GREEN, BLUE, RED = '#88dfbb', '#9ecfff', '#ffb0a5'


def cue_time(scene, phrase, normalize):
    words = scene['words']
    needle = normalize(phrase)
    for i in range(len(words)):
        value = ''
        for word in words[i:]:
            value += normalize(word['text'])
            if value == needle:
                return scene.get('speechLead',.25) + words[i]['offset'] / 1e7
            if not needle.startswith(value):
                break
    raise ValueError(f"Missing narration cue: {phrase!r} in {scene['title']}")


def prepare(scene, renderer):
    scene['cueTime'] = cue_time(scene, scene['cue'], renderer.speech_text) if scene.get('cue') else .4
    scene['finalCueTime'] = cue_time(scene, scene['finalCue'], renderer.speech_text) if scene.get('finalCue') else None
    scene['highlights'] = [cue_time(scene, phrase, renderer.speech_text) for phrase in scene.get('cues', [])]
    if scene['mode']=='cards':
        scene['highlightAt']=scene['cueTime']
    if scene['mode']=='deal':
        scene['dealEvents']=timeline.deal_timeline(scene['cueTime'],scene['before'],scene['after'],len(scene.get('table',{}).get('players',[0,1,2])),scene.get('folded',[]))
    end=0
    for action in scene.get('actions',[]):
        action['at']=max(end,cue_time(scene,action['cue'],renderer.speech_text))
        end=action['at']+timeline.ENTRY+.15
    scene['visualEnd']=max([end]+[e['end']+.5 for e in scene.get('dealEvents',[])])


def text(d, xy, value, size, renderer, color=INK, bold=False, width=1152):
    chosen = renderer.fit_text(d, value, size, width, bold)
    d.text(xy, value, font=chosen, fill=color)


@lru_cache(maxsize=80)
def small_card(code, renderer, size=100):
    suit = {'h':'♥','d':'♦','s':'♠','c':'♣'}[code[-1]]
    return renderer.card('10' if code[0]=='T' else code[0], suit).resize((size,round(size*1.42)),Image.Resampling.LANCZOS)


def chip(d,x,y,color=GOLD):
    d.ellipse((x-23,y-11,x+23,y+11),fill='#0c211b',outline=color,width=3)
    d.ellipse((x-23,y-16,x+23,y+6),fill=color,outline=INK,width=2)
    d.ellipse((x-12,y-11,x+12,y+1),outline='#294d3c',width=2)


def table(d):
    d.rounded_rectangle((48,196,1232,596),radius=165,fill='#153c30',outline='#668471',width=3)
    d.rounded_rectangle((63,211,1217,581),radius=151,outline='#315845',width=2)


def row_cards(im,d,codes,y,selected,renderer,t,invalid=False):
    width=100; gap=24; x0=(WIDTH-(len(codes)*(width+gap)-gap))/2
    for i,code in enumerate(codes):
        ease=1-(1-min(1,max(0,(t-.3-i*.16)/.65)))**3
        x=round(x0+i*(width+gap)); yy=round(y+(1-ease)*45)
        if ease<=0: continue
        im.paste(small_card(code,renderer),(x,yy),small_card(code,renderer))
        if i in selected and t>=1.8:
            d.rounded_rectangle((x-5,yy-5,x+104,yy+146),radius=12,outline=RED if invalid else GREEN,width=5)


def frame(scene,t,index,count,renderer):
    im=Image.new('RGB',(WIDTH,HEIGHT),BG); d=ImageDraw.Draw(im)
    text(d,(48,24),'ZT / ZERO TILT ACADEMY',20,renderer,GOLD,True)
    text(d,(890,24),f'{index+1:02}/{count:02}  •  BRAZILIAN PINEAPPLE',18,renderer,MUTED,width=345)
    d.line((48,66,1232,66),fill='#345044',width=2)
    text(d,(48,82),scene['chapter'].upper(),18,renderer,GREEN,True)
    text(d,(45,115),scene['title'],47,renderer,bold=True)
    mode=scene['mode']; focus=max(((a,i) for i,a in enumerate(scene['highlights']) if t>=a),default=(0,-1))[1]
    if mode in ('cards','deal','bets'):
        panel=timeline.table_frame(scene,t,renderer)
        im.paste(panel.resize((1184,407),Image.Resampling.LANCZOS),(48,196))
    elif mode in ('cards_legacy','deal_legacy'):
        table(d)
        if mode=='deal':
            stage=scene['stages'][int(t>=scene['cueTime'])]
            hole=['Ah','Kd','8c','6s','2h'][:stage]
            board=['Qc','Js','Th','4d','3c'][:(0 if stage==2 else stage)]
            text(d,(83,213),f'{stage} PRIVADAS',23,renderer,GOLD,True)
            text(d,(83,394),f'{len(board)} COMUNITÁRIAS',23,renderer,MUTED,True)
            row_cards(im,d,hole,246,[],renderer,t if t<scene['cueTime'] else t-scene['cueTime']+.3)
            row_cards(im,d,board,426,[],renderer,t)
        else:
            text(d,(80,207),'SUAS CARTAS',22,renderer,GOLD,True)
            text(d,(80,395),'MESA',22,renderer,MUTED,True)
            row_cards(im,d,scene['hole'],243,scene.get('selectHole',[]),renderer,t,scene.get('invalid',False))
            row_cards(im,d,scene['board'],428,scene.get('selectBoard',[]),renderer,t,scene.get('invalid',False))
    elif mode in ('intro','outro'):
        table(d)
        row_cards(im,d,['Ah','Kd','8c','6s','2h'],257,[0,1],renderer,t)
        text(d,(150,432),scene['lines'][0],45,renderer,GOLD,True,width=1000)
        text(d,(150,495),scene['lines'][1],30,renderer,MUTED,width=1000)
        if len(scene['lines'])>2: text(d,(150,541),scene['lines'][2],28,renderer,GREEN,width=1000)
    elif mode=='levels':
        for i,val in enumerate(scene['values']):
            x=55+310*i; active=focus==i
            d.rounded_rectangle((x,250,x+285,494),radius=18,fill='#284d3c' if active else '#142d25',outline=GOLD if active else '#456153',width=3)
            text(d,(x+27,282),f'{i+1}º PATAMAR',25,renderer,GREEN,True)
            text(d,(x+35,346),val,68,renderer,GOLD,True)
            for c in range(i+1): chip(d,x+80+c*37,463)
        text(d,(65,542),'BB já é o primeiro. Só três aumentos completos.',33,renderer)
    elif mode=='reopen':
        n=len(scene['rows']); h=82 if n==4 else 136
        for i,(name,value,status) in enumerate(scene['rows']):
            y=215+i*(h+12); active=focus==i
            d.rounded_rectangle((56,y,1224,y+h),radius=12,fill='#254b3c' if active else '#142e25',outline=GOLD if active else '#3c5e4d',width=3)
            text(d,(80,y+(h-49)//2),name,36,renderer,GOLD,True,width=215)
            text(d,(305,y+(h-55)//2),value,42,renderer,BLUE,True,width=335)
            text(d,(665,y+(h-46)//2),status,34,renderer,GREEN if 'reabre' in status else INK,True,width=530)
    elif mode in ('money','totals','pot','sidepots'):
        potmode=mode=='pot'; y=370 if potmode else 270
        if potmode:
            phase=min(1,max(0,(t-scene['cueTime'])/.85))
            amount=round(scene['before']+(scene['after']-scene['before'])*phase)
            if scene['finalCueTime'] is not None and t>=scene['finalCueTime']:
                phase=min(1,(t-scene['finalCueTime'])/.85)
                amount=round(scene['after']+(scene['final']-scene['after'])*phase)
            text(d,(60,223),'POTE AGORA',26,renderer,GOLD,True)
            text(d,(60,261),f'R$ {amount}',65,renderer,GOLD,True)
            for c in range(6): chip(d,465+c*57-round((1-phase)*80),290+round((1-phase)*45))
        colors=[GOLD,BLUE,GREEN]
        for i,(label,value) in enumerate(scene['metrics']):
            x=48+i*405; active=focus==i
            d.rounded_rectangle((x,y,x+377,y+179),radius=16,fill='#214738' if active else '#142e26',outline=colors[i],width=4 if active else 2)
            text(d,(x+22,y+21),label,23,renderer,colors[i],True,width=335)
            text(d,(x+22,y+72),value,61,renderer,colors[i],True,width=333)
        if mode=='sidepots':
            for c in range(9): chip(d,230+c*100,516,colors[c//3])
    else:
        lines=scene.get('lines',[])
        h=min(110,340//max(len(lines),1))
        for i,line in enumerate(lines):
            y=226+i*(h+14); active=focus==i
            d.rounded_rectangle((55,y,1225,y+h),radius=14,fill='#234c3a' if active else '#142e25',outline=GOLD if active else '#3f5b4d',width=3)
            text(d,(85,y+(h-51)//2),line,39,renderer,GOLD if active else INK,True,width=1115)
    text(d,(48,619),scene.get('note',''),27,renderer,RED if scene.get('invalid') else GOLD,True,width=1184)
    text(d,(48,675),'EXEMPLOS ILUSTRATIVOS  •  NARRAÇÃO SINTÉTICA  •  18+',16,renderer,MUTED)
    d.rectangle((0,713,round(WIDTH*min(1,t/scene['duration'])),719),fill=GREEN)
    return im


def poster(renderer):
    scene={'chapter':'Do primeiro blind ao showdown','title':'Aprenda Brazilian Pineapple','mode':'intro',
           'lines':['2 privadas + 3 comunitárias','Cartas, apostas e potes • Regras da modalidade'],
           'note':'SEM DESCARTE  •  ANTE 1 BB  •  AUMENTO ATÉ O POTE ANTES DO CALL',
           'highlights':[],'duration':1}
    return frame(scene,3,0,1,renderer)
