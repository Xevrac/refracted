using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SkillTreeUnlocks
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SkillTreeUnlocks); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SkillTreeUnlocks)obj;
            //  Serialize SkillTreeActive
            s.Write(value.SkillTreeActive);
            //  Serialize array NodesUnlocked
            Rts.Serialization.Reference.Write(s, value.NodesUnlocked, () =>
            {
                s.WriteVarInt32(value.NodesUnlocked.Length);
                for(int i = 0 ; i < value.NodesUnlocked.Length ; ++i)
                {
                    s.Write(value.NodesUnlocked[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SkillTreeUnlocks)) as Rts.CnC.Messages.Client.SkillTreeUnlocks;
            //  Deserialize SkillTreeActive
            s.Read(out value.SkillTreeActive);
            //  Deserialize array NodesUnlocked
            Rts.Serialization.Reference.Read(s, out value.NodesUnlocked, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
