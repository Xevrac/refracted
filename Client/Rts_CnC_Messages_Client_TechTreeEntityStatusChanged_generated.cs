using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeEntityStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeEntityStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeEntityStatusChanged)obj;
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize Unlocked
            s.Write(value.Unlocked);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeEntityStatusChanged)) as Rts.CnC.Messages.Client.TechTreeEntityStatusChanged;
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize Unlocked
            s.Read(out value.Unlocked);

            return value;
        }
        
    }
}
