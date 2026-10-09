"""Shared, deterministic table choreography in seconds and integer cents.

Existing cards are immutable; only new community cards enter face down and flip.
Private extras start two seconds after the final community reveal, including all-ins.
"""
from functools import lru_cache
from math import cos, pi
from PIL import Image, ImageDraw

ENTRY, CLOSED, FLIP, PAUSE, STAGGER = .6, 1., .8, 2., .35
VERSION = 2


def deal_timeline(start=0., before=2, after=5, players=3, folded=()):
    events = []
    cursor = start
    for stage in range(before + 1, after + 1):
        indexes = range(3) if stage == 3 else [stage - 1]
        community = []
        for j, index in enumerate(indexes):
            at = cursor + j * STAGGER
            event = dict(kind='board', index=index, start=at, land=at+ENTRY,
                         flip=at+ENTRY+CLOSED, end=at+ENTRY+CLOSED+FLIP)
            events.append(event); community.append(event)
        cursor = max(e['end'] for e in community) + PAUSE
        for player in range(players):
            if player in folded:
                continue
            events.append(dict(kind='hole', player=player, index=stage-1,
                               start=cursor, land=cursor+ENTRY, end=cursor+ENTRY))
            cursor += STAGGER
        cursor = max(e['end'] for e in events) + .5
    return events


def card_state(event, t):
    if t < event['start']:
        return None
    progress = min(1., (t-event['start'])/ENTRY)
    face = event['kind'] == 'hole' or t >= event['flip'] + FLIP/2
    scale = abs(cos(pi * min(1., max(0., (t-event.get('flip',t-FLIP))/FLIP)))) if event['kind']=='board' else 1.
    return dict(progress=1-(1-progress)**3, face=face, scale=max(.035,scale))


def money(value):
    return f'R$ {value/100:.2f}'.replace('.', ',')


def ledger(initial, actions, t):
    """Animation and assertions use the same ledger. Targets are street totals."""
    state = {**initial, 'players':[dict(p) for p in initial['players']]}
    state['payment'] = 0; state['actor'] = initial.get('actor',0)
    state['motion'] = None
    for action in actions:
        if t < action['at']:
            break
        p = state['players'][action['player']]
        amount = action.get('ante', action.get('to',p.get('bet',0))-p.get('bet',0))
        assert 0 <= amount <= p['stack'], action
        phase = min(1., (t-action['at'])/ENTRY)
        paid = round(amount*phase)
        p['stack'] -= paid
        if 'ante' not in action:
            p['bet'] = p.get('bet',0)+paid
        state['pot'] += paid
        state['actor'] = action['player']; state['payment'] = amount
        state['action'] = action.get('label','Pagamento')
        if phase < 1:
            state['motion'] = (action['player'],phase)
            break
        state['actor'] = action.get('next', (action['player']+1)%len(state['players']))
    return state


@lru_cache(maxsize=512)
def face_card(code, width, renderer, back=False):
    suit = {'h':'♥','d':'♦','s':'♠','c':'♣'}
    return renderer.card('10' if code[:1]=='T' else code[:1],suit.get(code[-1:],'♠'),back=back).resize((width,round(width*1.42)),Image.Resampling.LANCZOS)


def table_frame(scene, t, renderer):
    """1280×440 panel shared by film, institutional and technical lessons."""
    compact=scene.get('compact',False)
    canvas=(720,660) if compact else (1280,440)
    im=Image.new('RGB',canvas,'#0b1715'); d=ImageDraw.Draw(im)
    d.rounded_rectangle((10,8,canvas[0]-10,canvas[1]-4),radius=60,fill='#153c30',outline='#668471',width=3)
    initial=scene.get('table',{'pot':400,'players':[
        {'name':'Ana · você','stack':9900,'bet':0},
        {'name':'Beto','stack':9900,'bet':0},
        {'name':'Caio','stack':9800,'bet':0}]})
    state=ledger(initial,scene.get('actions',[]),t)
    players=state['players']; n=len(players)
    folded=scene.get('folded',[]); allin=scene.get('allin',[])
    before=scene.get('before',2); after=scene.get('after',before)
    events=scene.get('dealEvents',deal_timeline(scene.get('dealStart',.5),before,after,n,folded))
    if not scene.get('extra',True): events=[e for e in events if e['kind']=='board']
    holes=scene.get('hole',['Ah','Kd','8c','6s','2h'])
    board=scene.get('board',['Qc','Js','Th','4d','3c'])
    old_board=0 if before==2 else before
    # Explanatory hands are already on the table, clearly identified as a new example.
    if scene.get('snapshot'):
        old_board=len(board); before=len(holes); events=[]
    def txt(x,y,value,size=24,color='#f7f1e3'):
        d.text((x,y),value,font=renderer.font(size,True),fill=color)
    positions=([(210,490),(30,32),(375,32)] if compact else [(470,300),(45,25),(910,25),(910,300),(45,300),(475,25)])[:n]
    for j,p in enumerate(players):
        x,y=positions[j]
        active=state['actor']==j and not scene.get('noTurn')
        d.rounded_rectangle((x-10,y-8,x+335,y+126),radius=12,fill='#102820',outline='#e1c17a' if active else '#466b58',width=3 if active else 1)
        txt(x,y,p['name']+(' · VEZ' if active else ''),23,'#e1c17a')
        txt(x,y+29,f"Saldo {money(p['stack'])} · rodada {money(p.get('bet',0))}",19)
        if j==scene.get('dealer',0):
            d.ellipse((x-42,y,x-14,y+28),fill='#eee8d9'); d.text((x-36,y+2),'D',font=renderer.font(19,True),fill='#123226')
        if j in folded: txt(x+228,y+58,'FOLD',20,'#ffb0a5')
        elif j in allin: txt(x+226,y+58,'ALL-IN',18,'#9ecfff')
        width=42; limit=min(scene.get('initialHole',before),len(holes))
        for k in range(limit):
            c=face_card(holes[k],width,renderer,back=j!=0)
            im.paste(c,(x+k*47,y+58),c)
        for event in events:
            if event['kind']!='hole' or event['player']!=j: continue
            status=card_state(event,t)
            if status is None: continue
            code=holes[event['index']] if event['index']<len(holes) else '2h'
            c=face_card(code,width,renderer,back=j!=0)
            tx,ty=x+event['index']*47,y+58
            progress=status['progress']
            im.paste(c,(round(800+(tx-800)*progress),round(190+(ty-190)*progress)),c)
    for k,code in enumerate(board):
        event=next((e for e in events if e['kind']=='board' and e['index']==k),None)
        if k>=old_board and event is None: continue
        status=card_state(event,t) if event else dict(progress=1,face=True,scale=1)
        if status is None: continue
        width=86 if compact else 66; height=round(width*1.42)
        c=face_card(code,width,renderer,back=not status['face'])
        cw=max(2,round(width*status['scale'])); c=c.resize((cw,height),Image.Resampling.LANCZOS)
        progress=status['progress']; tx=(120 if compact else 450)+k*(width+10)+(width-cw)//2; ty=245 if compact else 174
        im.paste(c,(round(800+(tx-800)*progress),round(110+(ty-110)*progress)),c)
        if k in scene.get('selectBoard',[]) and t>=scene.get('highlightAt',3):
            d.rounded_rectangle((tx-3,ty-3,tx+cw+3,ty+height+3),radius=6,outline='#ffb0a5' if scene.get('invalid') else '#88dfbb',width=3)
    if scene.get('snapshot'):
        for k in scene.get('selectHole',[]):
            x,y=positions[0]
            if t>=scene.get('highlightAt',3): d.rounded_rectangle((x+k*47-3,y+55,x+k*47+45,y+120),radius=6,outline='#ffb0a5' if scene.get('invalid') else '#88dfbb',width=3)
    txt(238 if compact else 478,195 if compact else 128,'POTE '+money(state['pot']),28,'#e1c17a')
    txt(35 if compact else 45,395 if compact else 174,scene.get('example','EXEMPLO INDEPENDENTE'),17,'#88dfbb')
    if scene.get('actions'):
        txt(45,212,state.get('action','Aguardando ação'),20)
        txt(45,242,'Paga agora '+money(state['payment']),20,'#e1c17a')
    elif events:
        started=[e for e in events if t>=e['start']]
        last=started[-1] if started else None
        message='Privadas extras' if last and last['kind']=='hole' else 'Comunitária → revelação → pausa de 2s'
        txt(35 if compact else 45,432 if compact else 213,message,17)
    if state['motion']:
        who,phase=state['motion']; x,y=positions[who]
        x=round(x+140+(650-x-140)*phase); y=round(y+50+(145-y-50)*phase)
        for k in range(3): d.ellipse((x-14,y-5-k*5,x+14,y+7-k*5),fill='#e1c17a',outline='#f7f1e3',width=2)
    return im
