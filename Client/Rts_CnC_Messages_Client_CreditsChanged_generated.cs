using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_CreditsChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.CreditsChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.CreditsChanged)obj;
            //  Serialize Type
            s.Write(value.Type);
            //  Serialize Value
            s.Write(value.Value);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.CreditsChanged)) as Rts.CnC.Messages.Client.CreditsChanged;
            //  Deserialize Type
            s.Read(out value.Type);
            //  Deserialize Value
            s.Read(out value.Value);

            return value;
        }
        
    }
}
