using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestPatrolGroup
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestPatrolGroup); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestPatrolGroup)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array EntityIds
            Rts.Serialization.Reference.Write(s, value.EntityIds, () =>
            {
                s.WriteVarInt32(value.EntityIds.Length);
                for(int i = 0 ; i < value.EntityIds.Length ; ++i)
                {
                    s.Write(value.EntityIds[i]);
                }
            });
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestPatrolGroup)) as Rts.CnC.Messages.Client.RequestPatrolGroup;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array EntityIds
            Rts.Serialization.Reference.Read(s, out value.EntityIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
