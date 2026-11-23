"""Blender: save the editable neutral rig; render mouth/deformation witnesses.
blender -b -t4 --python scripts/preview_worm_a.py -- NEW_OUTPUT_DIRECTORY
"""
import bpy,math,sys
from mathutils import Vector,Quaternion
from pathlib import Path
root=Path(__file__).resolve().parents[1];out=Path(sys.argv[sys.argv.index('--')+1]);out.mkdir(parents=True,exist_ok=False)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(root/'assets/creatures/worm_a/Worm_A_rigged.glb'))
arm=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE');arm.show_in_front=True
print('RIG_BONES',len(arm.data.bones),[b.name for b in arm.data.bones]);bpy.context.view_layer.objects.active=arm
# Save the actual authoring asset without studio objects or animation clips.
bpy.ops.wm.save_as_mainfile(filepath=str(root/'assets/creatures/worm_a/Worm_A_rigged.blend'),compress=True)
points=[o.matrix_world@v.co for o in bpy.context.scene.objects if o.type=='MESH' for v in o.data.vertices]
lo=Vector([min(p[i] for p in points) for i in range(3)]);hi=Vector([max(p[i] for p in points) for i in range(3)]);center=(lo+hi)/2;extent=max(hi-lo)
world=bpy.context.scene.world=bpy.data.worlds.new('Studio');world.use_nodes=True;world.node_tree.nodes['Background'].inputs[0].default_value=(.06,.065,.06,1)
for pos,power in [((3,-4,5),500),((-4,1,2),300),((0,4,4),500)]:
 bpy.ops.object.light_add(type='AREA',location=center+Vector(pos)*extent*.5);o=bpy.context.object;o.data.energy=power;o.data.size=extent*2;o.rotation_euler=(center-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add();cam=bpy.context.object;bpy.context.scene.camera=cam;cam.data.type='ORTHO';cam.data.ortho_scale=extent*1.25
# Blender import maps canonical glTF+Y to Blender+Z: inspect that actual mouth side.
cam.location=center+Vector((.25,-.2,1.2)).normalized()*extent*3;cam.rotation_euler=(center-cam.location).to_track_quat('-Z','Y').to_euler()
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=24;scene.render.resolution_x=900;scene.render.resolution_y=900;scene.render.resolution_percentage=100
for pose,angle in [('neutral',0),('closed',-.25),('open',.75),('bend',0)]:
 for bone in arm.pose.bones:bone.rotation_mode='QUATERNION';bone.rotation_quaternion=Quaternion()
 for name,sign in [('jaw_left',-1),('jaw_right',1)]:
  bone=arm.pose.bones[name];basis=(arm.matrix_world@bone.bone.matrix_local).to_quaternion();axis=basis.inverted()@Vector((0,-1,0));bone.rotation_quaternion=Quaternion(axis,sign*angle)
 for obj in bpy.context.scene.objects:
  if obj.type=='MESH' and obj.data.shape_keys:
   for key in list(obj.data.shape_keys.key_blocks)[1:]:key.value=1 if pose=='closed' else 0
 if pose=='bend':
  for i in range(12):
   bone=arm.pose.bones[f'spine_{i:02}'];basis=(arm.matrix_world@bone.bone.matrix_local).to_quaternion();bone.rotation_quaternion=Quaternion(basis.inverted()@Vector((0,-1,0)),.12*math.sin(i*.8))
 scene.render.filepath=str(out/(pose+'.png'));bpy.ops.render.render(write_still=True)
 saved=cam.location.copy();rotation=cam.rotation_euler.copy();zoom=cam.data.ortho_scale
 target=Vector((0,-.28,.65));cam.location=target+Vector((.08,-.12,1.7));cam.rotation_euler=(target-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.ortho_scale=1.0
 scene.render.filepath=str(out/('mouth_'+pose+'.png'));bpy.ops.render.render(write_still=True)
 cam.location=saved;cam.rotation_euler=rotation;cam.data.ortho_scale=zoom
