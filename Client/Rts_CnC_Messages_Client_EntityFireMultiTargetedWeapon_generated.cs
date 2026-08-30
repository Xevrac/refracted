using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityFireMultiTargetedWeapon
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityFireMultiTargetedWeapon); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityFireMultiTargetedWeapon)obj;
            //  Serialize array Targets
            Rts.Serialization.Reference.Write(s, value.Targets, () =>
            {
                s.WriteVarInt32(value.Targets.Length);
                for(int i = 0 ; i < value.Targets.Length ; ++i)
                {
                    s.Write(value.Targets[i]);
                }
            });
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityFireMultiTargetedWeapon)) as Rts.CnC.Messages.Client.EntityFireMultiTargetedWeapon;
            //  Deserialize array Targets
            Rts.Serialization.Reference.Read(s, out value.Targets, () =>
            {
                int length = s.ReadVarInt32();
                SlimMath.Vector3[] tmp = new SlimMath.Vector3[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);

            return value;
        }
        
    }
}
